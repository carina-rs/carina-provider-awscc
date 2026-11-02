//! CloudFormation input model and shared property-path predicates.

use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

/// CloudFormation Resource Schema
#[derive(Debug, Clone, Deserialize)]
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
    /// Top-level allOf constraints also apply to the properties they mention.
    #[serde(default, rename = "allOf")]
    pub(super) all_of: Vec<CfnProperty>,
    /// Cloud Control handlers (create/read/update/delete/list). Most
    /// NON_PROVISIONABLE types omit this map, but the registry has known
    /// provisionable exceptions; see codegen's `HANDLERLESS_CLOUD_CONTROL_TYPES`.
    #[serde(default)]
    pub(super) handlers: BTreeMap<String, serde_json::Value>,
}

/// Type can be a string or an array of strings in JSON Schema
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
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
    #[serde(rename = "const", default, deserialize_with = "deserialize_const")]
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
    #[serde(default, rename = "allOf")]
    pub(super) all_of: Vec<CfnProperty>,
}

// A present `const: null` is a constraint, unlike an absent const keyword.
fn deserialize_const<'de, D>(deserializer: D) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_json::Value::deserialize(deserializer).map(Some)
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
#[derive(Debug, Clone, Deserialize)]
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

/// The final definition reached by a local `$ref` chain.
///
/// Definitions use the exact same representation as inline properties, so
/// every keyword supported by `CfnProperty` automatically survives `$ref`
/// resolution. `name` is the final target's name and is used only for stable
/// struct identity and recursion handling; shape generation always consumes
/// `property` through the normal inline path.
#[derive(Clone, Copy)]
pub(super) struct ResolvedRef<'a> {
    pub(super) name: &'a str,
    pub(super) property: &'a CfnProperty,
}

/// Resolve a local definition `$ref`, following definitions that are aliases
/// to other definitions. A malformed path, missing definition, external ref,
/// or alias cycle is unresolved rather than recursively looping.
pub(super) fn resolve_ref<'a>(schema: &'a CfnSchema, ref_path: &str) -> Option<ResolvedRef<'a>> {
    let definitions = schema.definitions.as_ref()?;
    let mut name = ref_def_name(ref_path)?;
    let mut visited = HashSet::new();

    loop {
        if !visited.insert(name) {
            return None;
        }
        let (definition_name, property) = definitions.get_key_value(name)?;
        let Some(next_ref) = property.ref_path.as_deref() else {
            return Some(ResolvedRef {
                name: definition_name,
                property,
            });
        };
        name = ref_def_name(next_ref)?;
    }
}

pub(super) fn has_direct_struct_properties(property: &CfnProperty) -> bool {
    property
        .properties
        .as_ref()
        .is_some_and(|properties| !properties.is_empty())
}

/// Inline object properties are authoritative even when a schema omits the
/// redundant `type: object` marker. This preserves the old definition
/// precedence while keeping definitions and inline properties on one path.
pub(super) fn inline_property_type(property: &CfnProperty) -> Option<&str> {
    if has_direct_struct_properties(property) {
        Some("object")
    } else {
        property
            .prop_type
            .as_ref()
            .and_then(TypeValue::effective_type)
    }
}

pub(super) enum PropertyShape<'a> {
    Reference {
        path: &'a str,
        resolved: Option<ResolvedRef<'a>>,
    },
    Inline,
    Direct(&'a BTreeMap<String, CfnProperty>),
    Array {
        /// Keep the winning variant's reference context for generation.
        variant: &'a CfnProperty,
        /// Reuse the concrete array without resolving nested unions again.
        resolved: &'a CfnProperty,
    },
    /// Union of object fields; variant-local required lists do not apply.
    Merged(BTreeMap<String, CfnProperty>),
    Unresolved,
}

pub(super) fn property_shape<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
) -> PropertyShape<'a> {
    property_shape_with_visited(property, schema, &mut HashSet::new())
}

/// The sole shape precedence rule: a reference owns the shape before any
/// sibling keywords. Inline properties then use direct fields, the first
/// oneOf variant resolving to an array, or merged optional object fields.
pub(super) fn property_shape_with_visited<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> PropertyShape<'a> {
    if let Some(path) = property.ref_path.as_deref() {
        return PropertyShape::Reference {
            path,
            resolved: resolve_ref(schema, path),
        };
    }
    if let Some(properties) = property
        .properties
        .as_ref()
        .filter(|props| !props.is_empty())
    {
        return PropertyShape::Direct(properties);
    }
    if property.one_of.is_empty() {
        return PropertyShape::Inline;
    }
    if let Some(shape) = property.one_of.iter().find_map(|variant| {
        resolved_array_property(variant, schema, visiting_defs)
            .map(|resolved| PropertyShape::Array { variant, resolved })
    }) {
        return shape;
    }
    let mut merged = BTreeMap::new();
    for variant in &property.one_of {
        merged.extend(resolved_object_properties(variant, schema, visiting_defs));
    }
    if merged.is_empty() {
        PropertyShape::Unresolved
    } else {
        PropertyShape::Merged(merged)
    }
}

/// Follow object variants with the same precedence and cycle guard as array
/// variants. A variant's required list does not apply to the merged object.
pub(super) fn resolved_object_properties<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> BTreeMap<String, CfnProperty> {
    match property_shape_with_visited(property, schema, visiting_defs) {
        PropertyShape::Reference {
            resolved: Some(resolved),
            ..
        } => {
            if !visiting_defs.insert(resolved.name) {
                return BTreeMap::new();
            }
            let properties = resolved_object_properties(resolved.property, schema, visiting_defs);
            visiting_defs.remove(resolved.name);
            properties
        }
        PropertyShape::Direct(properties) => properties.clone(),
        PropertyShape::Merged(properties) => properties,
        PropertyShape::Reference { resolved: None, .. }
        | PropertyShape::Inline
        | PropertyShape::Array { .. }
        | PropertyShape::Unresolved => BTreeMap::new(),
    }
}

/// Whether the inline generator will represent this property as a struct.
pub(super) fn property_generates_struct(property: &CfnProperty, schema: &CfnSchema) -> bool {
    match property_shape(property, schema) {
        PropertyShape::Reference { resolved, .. } => {
            resolved.is_some_and(|resolved| property_generates_struct(resolved.property, schema))
        }
        PropertyShape::Direct(_) | PropertyShape::Merged(_) => true,
        PropertyShape::Inline | PropertyShape::Array { .. } | PropertyShape::Unresolved => false,
    }
}

/// Resolve aliases and oneOf arrays through the same shape classifier used
/// for inline properties. Track definition visits because oneOf branches
/// can refer back to their containing definition.
pub(super) fn resolved_array_property<'a>(
    prop: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> Option<&'a CfnProperty> {
    match property_shape_with_visited(prop, schema, visiting_defs) {
        PropertyShape::Reference { resolved, .. } => {
            let resolved = resolved?;
            if !visiting_defs.insert(resolved.name) {
                return None;
            }
            let array = resolved_array_property(resolved.property, schema, visiting_defs);
            visiting_defs.remove(resolved.name);
            array
        }
        PropertyShape::Inline => (inline_property_type(prop) == Some("array")).then_some(prop),
        PropertyShape::Array { resolved, .. } => Some(resolved),
        PropertyShape::Direct(_) | PropertyShape::Merged(_) | PropertyShape::Unresolved => None,
    }
}

pub(super) fn has_nested_read_only_property_path(
    schema: &CfnSchema,
    property_path: &[String],
) -> bool {
    schema
        .read_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer))
        .any(|path| path.len() > property_path.len() && path.starts_with(property_path))
}

pub(super) fn struct_fields_are_fully_read_only(
    schema: &CfnSchema,
    properties: &BTreeMap<String, CfnProperty>,
    property_path: &[String],
    visiting_defs: &mut HashSet<String>,
) -> bool {
    let mut fields = properties
        .iter()
        .filter(|(_, property)| !is_deprecated_property(property))
        .peekable();
    fields.peek().is_some()
        && fields.all(|(field_name, field)| {
            let field_path = extend_property_path(property_path, field_name);
            property_is_fully_read_only(schema, field, &field_path, visiting_defs)
        })
}

pub(super) fn property_is_fully_read_only(
    schema: &CfnSchema,
    property: &CfnProperty,
    property_path: &[String],
    visiting_defs: &mut HashSet<String>,
) -> bool {
    if is_read_only_property_path(schema, property_path) {
        return true;
    }
    if !has_nested_read_only_property_path(schema, property_path) {
        return false;
    }

    match property_shape(property, schema) {
        PropertyShape::Reference { resolved, .. } => {
            let Some(resolved) = resolved else {
                return false;
            };
            if !visiting_defs.insert(resolved.name.to_string()) {
                return false;
            }
            let fully_read_only = property_is_fully_read_only(
                schema,
                resolved.property,
                property_path,
                visiting_defs,
            );
            visiting_defs.remove(resolved.name);
            return fully_read_only;
        }
        PropertyShape::Array {
            variant: array_variant,
            ..
        } => {
            return property_is_fully_read_only(
                schema,
                array_variant,
                property_path,
                visiting_defs,
            );
        }
        PropertyShape::Direct(properties) => {
            return struct_fields_are_fully_read_only(
                schema,
                properties,
                property_path,
                visiting_defs,
            );
        }
        PropertyShape::Merged(properties) => {
            return struct_fields_are_fully_read_only(
                schema,
                &properties,
                property_path,
                visiting_defs,
            );
        }
        PropertyShape::Unresolved => return false,
        PropertyShape::Inline => {}
    }

    match inline_property_type(property) {
        Some("object") => property.properties.as_ref().is_some_and(|properties| {
            struct_fields_are_fully_read_only(schema, properties, property_path, visiting_defs)
        }),
        Some("array") => property.items.as_deref().is_some_and(|items| {
            let item_path = extend_property_path(property_path, "*");
            property_is_fully_read_only(schema, items, &item_path, visiting_defs)
        }),
        _ => false,
    }
}

pub(super) fn is_property_read_only(
    schema: &CfnSchema,
    property: &CfnProperty,
    property_path: &[String],
) -> bool {
    property_is_fully_read_only(schema, property, property_path, &mut HashSet::new())
}

pub(super) fn is_deprecated_property(prop: &CfnProperty) -> bool {
    prop.description
        .as_deref()
        .is_some_and(|description| description.starts_with("(Deprecated.)"))
}

/// Known enum overrides for properties where `extract_enum_from_description()` fails
/// due to inconsistent description formatting in CloudFormation schemas.
pub(super) fn known_enum_overrides() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static OVERRIDES: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("IpProtocol", vec!["tcp", "udp", "icmp", "icmpv6", "-1"]);
        m.insert("ConnectivityType", vec!["public", "private"]);
        m.insert("AvailabilityMode", vec!["zonal", "regional"]);
        m.insert("AddressFamily", vec!["IPv4", "IPv6"]);
        m.insert("Domain", vec!["vpc", "standard"]);
        // HostnameType enum values are in parent struct description, not field description
        m.insert("HostnameType", vec!["ip-name", "resource-name"]);
        // InternetGatewayBlockMode removed - now auto-detected via "Options here are" pattern
        // Transit gateway enable/disable properties
        m.insert("AutoAcceptSharedAttachments", vec!["enable", "disable"]);
        m.insert("DefaultRouteTableAssociation", vec!["enable", "disable"]);
        m.insert("DefaultRouteTablePropagation", vec!["enable", "disable"]);
        m.insert("DnsSupport", vec!["enable", "disable"]);
        m.insert("MulticastSupport", vec!["enable", "disable"]);
        m.insert("SecurityGroupReferencingSupport", vec!["enable", "disable"]);
        m.insert("VpnEcmpSupport", vec!["enable", "disable"]);
        m.insert("EncryptionSupportState", vec!["disable", "enable"]);
        // CloudFront ViewerCertificate.MinimumProtocolVersion: CFN schema declares
        // it as a free-form string, so the description-scraping heuristic in
        // extract_enum_from_description() picked up unrelated backticked tokens
        // from sibling fields (`sni-only` from SslSupportMethod, `true` from
        // CloudFrontDefaultCertificate) while filtering out the real value
        // `TLSv1` as a "property name". Pin the canonical security-policy values
        // here so the heuristic is short-circuited (overrides are consulted
        // before description extraction). awscc#242.
        m.insert(
            "MinimumProtocolVersion",
            vec![
                "SSLv3",
                "TLSv1",
                "TLSv1_2016",
                "TLSv1.1_2016",
                "TLSv1.2_2018",
                "TLSv1.2_2019",
                "TLSv1.2_2021",
            ],
        );
        m
    });
    &OVERRIDES
}

/// Shared scalar inference for classification and both renderers, including
/// their enum/const precedence and implicit string fallback. Supported
/// object|string unions project to a policy-document string before emission.
pub(super) fn emits_string_leaf(property: &CfnProperty, name: &str) -> bool {
    if property.const_value.is_some() {
        return true;
    }
    if let Some(values) = &property.enum_values {
        return values.is_empty()
            || !values
                .iter()
                .all(|value| matches!(value, EnumValue::Int(_)));
    }
    if known_enum_overrides().contains_key(name) {
        return true;
    }
    if property
        .prop_type
        .as_ref()
        .is_some_and(|types| types.contains("object") && types.contains("string"))
    {
        return true;
    }
    !matches!(
        inline_property_type(property),
        Some("boolean" | "integer" | "number" | "array" | "object")
    )
}

/// Whether this CloudFormation property is generated as the canonical DSL
/// `tags: Map<String, String>` attribute.
pub(super) fn is_dsl_tags_property(schema: &CfnSchema, provider_name: &str) -> bool {
    let Some(tagging) = schema.tagging.as_ref().filter(|tagging| tagging.taggable) else {
        return false;
    };
    let tag_property = tagging
        .tag_property
        .as_deref()
        .unwrap_or("/properties/Tags");

    tag_property.strip_prefix("/properties/") == Some(provider_name)
        && provider_name.to_snake_case() == "tags"
}

/// Whether a definition uses one of Carina's canonical tag-map shapes.
///
/// The final resolved definition's `Tag` prefix remains a namespace guard
/// because cached schemas also use the exact same Key/Value string-object
/// shape for non-tag ELB attributes. The shape check prevents unrelated
/// `Tag*` objects from silently losing fields when converted to Carina's map
/// representation. CloudFormation uses either a Key/Value object or a
/// single-pattern string map for these refs.
pub(super) fn is_tag_map_ref(schema: &CfnSchema, ref_path: &str) -> bool {
    let Some(resolved) = resolve_ref(schema, ref_path) else {
        return false;
    };
    let definition = resolved.property;
    let is_direct_string = |property: &CfnProperty| {
        property
            .prop_type
            .as_ref()
            .and_then(TypeValue::effective_type)
            == Some("string")
    };
    let is_key_value_object = definition.properties.as_ref().is_some_and(|properties| {
        properties.len() == 2
            && properties.get("Key").is_some_and(is_direct_string)
            && properties.get("Value").is_some_and(is_direct_string)
    });
    let is_single_pattern_string_map = definition.properties.is_none()
        && definition
            .pattern_properties
            .as_ref()
            .is_some_and(|patterns| patterns.len() == 1 && patterns.values().all(is_direct_string));

    resolved.name.starts_with("Tag")
        && definition
            .prop_type
            .as_ref()
            .and_then(TypeValue::effective_type)
            == Some("object")
        && (is_key_value_object || is_single_pattern_string_map)
}

/// Enum discovery never emits a composite body. Each reachable struct is
/// scanned separately at its own use path, including recursive definitions.
pub(super) fn has_scalar_leaf(property: &CfnProperty, schema: &CfnSchema) -> bool {
    fn visit<'a>(
        property: &'a CfnProperty,
        schema: &'a CfnSchema,
        visiting: &mut HashSet<&'a str>,
    ) -> bool {
        if property.const_value.is_some() {
            return true;
        }
        match property_shape(property, schema) {
            PropertyShape::Reference {
                resolved: Some(resolved),
                ..
            } => {
                if !visiting.insert(resolved.name) {
                    return false;
                }
                let scalar = visit(resolved.property, schema, visiting);
                visiting.remove(resolved.name);
                scalar
            }
            PropertyShape::Reference { resolved: None, .. } => true,
            PropertyShape::Array { resolved, .. } => resolved
                .items
                .as_deref()
                .is_some_and(|items| visit(items, schema, visiting)),
            PropertyShape::Direct(_) | PropertyShape::Merged(_) => false,
            PropertyShape::Inline | PropertyShape::Unresolved => {
                match inline_property_type(property) {
                    Some("array") => property
                        .items
                        .as_deref()
                        .is_some_and(|items| visit(items, schema, visiting)),
                    Some("object") => property
                        .pattern_properties
                        .iter()
                        .flat_map(|patterns| patterns.values())
                        .any(|value| visit(value, schema, visiting)),
                    _ => true,
                }
            }
        }
    }
    visit(property, schema, &mut HashSet::new())
}
