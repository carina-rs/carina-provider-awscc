//! Tag-only compatibility queries retained unchanged until STEP 5.
//! Shared non-tag queries must use schema_projection instead.
use super::{CfnProperty, CfnSchema, TypeValue};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};

pub(super) fn ref_def_name(ref_path: &str) -> Option<&str> {
    ref_path.strip_prefix("#/definitions/")
}

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
        resolved_array_shape(variant, schema, visiting_defs)
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

pub(super) fn resolved_object_properties<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> BTreeMap<String, CfnProperty> {
    resolved_object_property_sources(property, schema, visiting_defs)
        .into_iter()
        .map(|(name, property)| (name.to_string(), property.clone()))
        .collect()
}

pub(super) fn resolved_object_property_sources<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> BTreeMap<&'a str, &'a CfnProperty> {
    match property_shape_with_visited(property, schema, visiting_defs) {
        PropertyShape::Reference {
            resolved: Some(resolved),
            ..
        } => {
            if !visiting_defs.insert(resolved.name) {
                return BTreeMap::new();
            }
            let properties =
                resolved_object_property_sources(resolved.property, schema, visiting_defs);
            visiting_defs.remove(resolved.name);
            properties
        }
        PropertyShape::Direct(properties) => properties
            .iter()
            .map(|(name, property)| (name.as_str(), property))
            .collect(),
        PropertyShape::Merged(_) => {
            let mut properties = BTreeMap::new();
            for variant in &property.one_of {
                properties.extend(resolved_object_property_sources(
                    variant,
                    schema,
                    visiting_defs,
                ));
            }
            properties
        }
        PropertyShape::Reference { resolved: None, .. }
        | PropertyShape::Inline
        | PropertyShape::Array { .. }
        | PropertyShape::Unresolved => BTreeMap::new(),
    }
}

pub(super) fn resolved_array_shape<'a>(
    prop: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> Option<ResolvedArray<'a>> {
    match property_shape_with_visited(prop, schema, visiting_defs) {
        PropertyShape::Reference { path, .. } => {
            // Walk each alias rather than jumping to resolve_ref's final target:
            // every $ref sibling can contribute list metadata.
            let name = ref_def_name(path)?;
            if !visiting_defs.insert(name) {
                return None;
            }
            let array = schema
                .definitions
                .as_ref()
                .and_then(|definitions| definitions.get(name))
                .and_then(|target| resolved_array_shape(target, schema, visiting_defs));
            visiting_defs.remove(name);
            array.map(|array| array_with_ref_siblings(array, prop))
        }
        PropertyShape::Inline => {
            (inline_property_type(prop) == Some("array")).then_some(ResolvedArray::new(prop))
        }
        PropertyShape::Array { resolved, .. } => Some(array_with_ref_siblings(resolved, prop)),
        PropertyShape::Direct(_) | PropertyShape::Merged(_) | PropertyShape::Unresolved => None,
    }
}

pub(super) fn resolved_array_property<'a>(
    prop: &'a CfnProperty,
    schema: &'a CfnSchema,
    visiting_defs: &mut HashSet<&'a str>,
) -> Option<Cow<'a, CfnProperty>> {
    resolved_array_shape(prop, schema, visiting_defs).map(ResolvedArray::into_property)
}

pub(super) fn array_with_ref_siblings<'a>(
    mut array: ResolvedArray<'a>,
    sibling: &CfnProperty,
) -> ResolvedArray<'a> {
    array.min_items = match (array.min_items, sibling.min_items) {
        (Some(target), Some(sibling)) => Some(target.max(sibling)),
        (target, sibling) => target.or(sibling),
    };
    array.max_items = match (array.max_items, sibling.max_items) {
        (Some(target), Some(sibling)) => Some(target.min(sibling)),
        (target, sibling) => target.or(sibling),
    };
    array.unique_items = match (array.unique_items, sibling.unique_items) {
        (Some(target), Some(sibling)) => Some(target || sibling),
        (target, sibling) => target.or(sibling),
    };
    array.insertion_order = array.insertion_order.or(sibling.insertion_order);
    array
}

pub(super) fn is_string_property(schema: &CfnSchema, property: &CfnProperty) -> bool {
    if matches!(
        property
            .prop_type
            .as_ref()
            .and_then(TypeValue::effective_type),
        Some("string")
    ) {
        return true;
    }

    property
        .ref_path
        .as_deref()
        .and_then(|ref_path| resolve_ref(schema, ref_path))
        .is_some_and(|resolved| {
            resolved
                .property
                .prop_type
                .as_ref()
                .and_then(TypeValue::effective_type)
                == Some("string")
        })
}

pub(super) struct ResolvedRef<'a> {
    pub(super) name: &'a str,
    pub(super) property: &'a CfnProperty,
}

#[allow(dead_code)] // Retained tag compatibility shape; STEP 5 removes this walker.
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
        resolved: ResolvedArray<'a>,
    },
    /// Union of object fields; variant-local required lists do not apply.
    Merged(BTreeMap<String, CfnProperty>),
    Unresolved,
}

pub(super) struct ResolvedArray<'a> {
    pub(super) property: &'a CfnProperty,
    min_items: Option<i64>,
    max_items: Option<i64>,
    unique_items: Option<bool>,
    insertion_order: Option<bool>,
}

impl<'a> ResolvedArray<'a> {
    fn new(property: &'a CfnProperty) -> Self {
        Self {
            property,
            min_items: property.min_items,
            max_items: property.max_items,
            unique_items: property.unique_items,
            insertion_order: property.insertion_order,
        }
    }

    /// Materialize merged metadata only when a consumer needs a full property.
    pub(super) fn into_property(self) -> Cow<'a, CfnProperty> {
        let property = self.property;
        if (
            self.min_items,
            self.max_items,
            self.unique_items,
            self.insertion_order,
        ) == (
            property.min_items,
            property.max_items,
            property.unique_items,
            property.insertion_order,
        ) {
            Cow::Borrowed(property)
        } else {
            Cow::Owned(CfnProperty {
                min_items: self.min_items,
                max_items: self.max_items,
                unique_items: self.unique_items,
                insertion_order: self.insertion_order,
                ..property.clone()
            })
        }
    }
}

pub(super) fn property_shape<'a>(
    property: &'a CfnProperty,
    schema: &'a CfnSchema,
) -> PropertyShape<'a> {
    property_shape_with_visited(property, schema, &mut HashSet::new())
}

pub(super) fn property_generates_struct(property: &CfnProperty, schema: &CfnSchema) -> bool {
    match property_shape(property, schema) {
        PropertyShape::Reference { resolved, .. } => {
            resolved.is_some_and(|resolved| property_generates_struct(resolved.property, schema))
        }
        PropertyShape::Direct(_) | PropertyShape::Merged(_) => true,
        PropertyShape::Inline | PropertyShape::Array { .. } | PropertyShape::Unresolved => false,
    }
}

pub(super) fn is_struct_property(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    property_generates_struct(prop, schema)
}

pub(super) fn is_list_of_struct_property(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    resolved_array_property(prop, schema, &mut HashSet::new()).is_some_and(|property| {
        property
            .items
            .as_deref()
            .is_some_and(|items| is_struct_property(items, schema))
    })
}

/// Keep tag scalar projection at its existing scope until STEP 5. In particular
/// scalar allOf/dependency keywords must not start affecting tag admission here.
pub(super) fn scalar_projection(
    property: &CfnProperty,
) -> Result<super::Projection, super::schema_layer::SchemaError> {
    let origin = property
        .origin
        .as_ref()
        .expect("parsed tag scalar has an origin");
    super::schema_projection::direct(&origin.layer, origin.node)
}

pub(super) fn enum_definition_names(
    schema: &CfnSchema,
) -> anyhow::Result<std::collections::BTreeSet<String>> {
    let mut names = std::collections::BTreeSet::new();
    if let Some(path) = super::tag_property_path(schema)?
        && let Some(name) = path.strip_prefix("/properties/")
        && let Some(property) = schema.properties.get(name)
    {
        if let Some(reference) = property.ref_path.as_deref()
            && let Some(resolved) = resolve_ref(schema, reference)
        {
            names.insert(resolved.name.to_string());
        }
        if let Some(array) = resolved_array_property(property, schema, &mut HashSet::new())
            && let Some(items) = array.items.as_deref()
            && let Some(reference) = items.ref_path.as_deref()
            && let Some(resolved) = resolve_ref(schema, reference)
        {
            names.insert(resolved.name.to_string());
        }
    }
    Ok(names)
}

/// Definition enum bookkeeping is also tag-owned for these fields. Preserve
/// its old scalar scope, final ref identity, and literal spelling for STEP 5.
pub(super) fn enum_field(
    schema: &CfnSchema,
    property: &CfnProperty,
) -> Result<CfnProperty, super::schema_layer::SchemaError> {
    use super::schema_layer::JsonType;
    let reference = property
        .ref_path
        .as_deref()
        .and_then(|path| resolve_ref(schema, path).map(|resolved| (path, resolved)));
    let source = reference
        .as_ref()
        .map_or(property, |(_path, resolved)| resolved.property);
    let mut p = scalar_projection(source)?;
    p.types = match inline_property_type(source) {
        Some("null") | None => Vec::new(),
        Some("boolean") => vec![JsonType::Boolean],
        Some("object") => vec![JsonType::Object],
        Some("array") => vec![JsonType::Array],
        Some("number") => vec![JsonType::Number],
        Some("integer") => vec![JsonType::Integer],
        Some("string") => vec![JsonType::String],
        Some(_unknown) => Vec::new(),
    };
    fn spelling(value: serde_json::Value) -> String {
        match value {
            serde_json::Value::String(value) => value,
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::Bool(value) => value.to_string(),
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Array(values) => serde_json::Value::Array(values).to_string(),
            serde_json::Value::Object(values) => serde_json::Value::Object(values).to_string(),
        }
    }
    if let Some(value) = p.const_value.take() {
        p.const_value = Some(serde_json::Value::String(spelling(value)));
    }
    if let Some(values) = &mut p.enum_values
        && !values.iter().all(|value| value.as_i64().is_some())
    {
        *values = values
            .iter()
            .cloned()
            .map(|value| serde_json::Value::String(spelling(value)))
            .collect();
    }
    if let Some((path, resolved)) = reference {
        p.reference = Some(super::schema_projection::Reference {
            path: path.to_string(),
            name: resolved.name.to_string(),
            property: Box::new(super::schema_projection::materialize(p.clone())),
        });
    }
    Ok(super::schema_projection::materialize(p))
}
