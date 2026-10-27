//! Tag conversion helpers between the DSL map and schema-declared CloudControl wire formats.

use indexmap::IndexMap;

use carina_core::provider::PatchOp;
use carina_core::resource::{ConcreteValue, Value};
use serde_json::json;

use crate::schemas::config::{TagsProperty, TagsWire};

pub(super) fn wire_tags_are_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(tags) => tags.is_empty(),
        serde_json::Value::Object(tags) => tags.is_empty(),
        _ => false,
    }
}

impl TagsProperty {
    /// Convert a concrete DSL `Map<String, String>` into this property's wire shape.
    pub(crate) fn encode_dsl_tags(&self, value: &Value) -> Option<serde_json::Value> {
        let Value::Concrete(ConcreteValue::Map(tags)) = value else {
            return None;
        };
        let entries = tags
            .iter()
            .map(|(key, value)| match value {
                Value::Concrete(ConcreteValue::String(value)) => {
                    Some((key.as_str(), value.expose_plaintext()))
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;

        Some(match self.wire {
            TagsWire::KeyValueList => serde_json::Value::Array(
                entries
                    .into_iter()
                    .map(|(key, value)| json!({"Key": key, "Value": value}))
                    .collect(),
            ),
            TagsWire::Map => serde_json::Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key.to_string(),
                            serde_json::Value::String(value.to_string()),
                        )
                    })
                    .collect(),
            ),
        })
    }

    /// Convert this property's wire shape into the canonical DSL tag map.
    pub(crate) fn decode_wire_tags(
        &self,
        value: Option<&serde_json::Value>,
    ) -> IndexMap<String, Value> {
        let mut tags_map = IndexMap::new();
        match self.wire {
            TagsWire::KeyValueList => {
                if let Some(tags) = value.and_then(serde_json::Value::as_array) {
                    for tag in tags {
                        if let (Some(key), Some(value)) = (
                            tag.get("Key").and_then(serde_json::Value::as_str),
                            tag.get("Value").and_then(serde_json::Value::as_str),
                        ) {
                            insert_tag(&mut tags_map, key, value);
                        }
                    }
                }
            }
            TagsWire::Map => {
                if let Some(tags) = value.and_then(serde_json::Value::as_object) {
                    for (key, value) in tags {
                        if let Some(value) = value.as_str() {
                            insert_tag(&mut tags_map, key, value);
                        }
                    }
                }
            }
        }
        tags_map
    }

    /// Convert a tags operation to the complete CloudControl JSON Patch operation.
    ///
    /// Returning `None` means the DSL value could not be converted without loss.
    pub(crate) fn update_patch(&self, op: &PatchOp) -> Option<serde_json::Value> {
        match op {
            PatchOp::Add { value, .. }
            | PatchOp::Replace { value, .. }
            | PatchOp::Reassert { value, .. } => {
                let wire_value = self.encode_dsl_tags(value)?;
                if wire_tags_are_empty(&wire_value) {
                    Some(json!({"op": "remove", "path": format!("/{}", self.provider_name)}))
                } else {
                    Some(json!({
                        "op": "add",
                        "path": format!("/{}", self.provider_name),
                        "value": wire_value,
                    }))
                }
            }
            PatchOp::Remove { .. } => {
                Some(json!({"op": "remove", "path": format!("/{}", self.provider_name)}))
            }
        }
    }
}

fn insert_tag(tags: &mut IndexMap<String, Value>, key: &str, value: &str) {
    tags.insert(
        key.to_string(),
        Value::Concrete(ConcreteValue::String(value.to_string().into())),
    );
}

#[cfg(test)]
mod tests {
    use carina_core::provider::PatchOp;
    use serde_json::json;

    use super::*;
    use crate::schemas::config::{TagsProperty, TagsWire};

    fn tags_property(wire: TagsWire) -> TagsProperty {
        TagsProperty {
            provider_name: "Tags",
            wire,
        }
    }

    fn dsl_tags(entries: &[(&str, &str)]) -> Value {
        Value::Concrete(ConcreteValue::Map(
            entries
                .iter()
                .map(|(key, value)| {
                    (
                        (*key).to_string(),
                        Value::Concrete(ConcreteValue::String((*value).to_string().into())),
                    )
                })
                .collect(),
        ))
    }

    fn add(value: Value) -> PatchOp {
        PatchOp::Add {
            key: "tags".to_string(),
            value,
        }
    }

    fn replace(value: Value) -> PatchOp {
        PatchOp::Replace {
            key: "tags".to_string(),
            value,
        }
    }

    fn reassert(value: Value) -> PatchOp {
        PatchOp::Reassert {
            key: "tags".to_string(),
            value,
        }
    }

    fn remove() -> PatchOp {
        PatchOp::Remove {
            key: "tags".to_string(),
        }
    }

    #[test]
    fn key_value_list_converts_both_directions() {
        let property = tags_property(TagsWire::KeyValueList);
        let dsl = dsl_tags(&[("env", "dev"), ("owner", "carina")]);
        let wire = json!([
            {"Key": "env", "Value": "dev"},
            {"Key": "owner", "Value": "carina"}
        ]);

        assert_eq!(property.encode_dsl_tags(&dsl), Some(wire.clone()));
        assert_eq!(
            property.decode_wire_tags(Some(&wire)),
            match dsl {
                Value::Concrete(ConcreteValue::Map(tags)) => tags,
                _ => unreachable!(),
            }
        );
    }

    #[test]
    fn map_converts_both_directions() {
        let property = tags_property(TagsWire::Map);
        let dsl = dsl_tags(&[("env", "dev"), ("owner", "carina")]);
        let wire = json!({"env": "dev", "owner": "carina"});

        assert_eq!(property.encode_dsl_tags(&dsl), Some(wire.clone()));
        assert_eq!(
            property.decode_wire_tags(Some(&wire)),
            match dsl {
                Value::Concrete(ConcreteValue::Map(tags)) => tags,
                _ => unreachable!(),
            }
        );
    }

    #[test]
    fn map_read_of_absent_tags_is_empty() {
        let property = tags_property(TagsWire::Map);

        assert!(property.decode_wire_tags(None).is_empty());
    }

    #[test]
    fn update_patch_uses_each_wire_shape_for_add_replace_and_reassert() {
        for (wire, expected_value) in [
            (
                TagsWire::KeyValueList,
                json!([{"Key": "env", "Value": "prod"}]),
            ),
            (TagsWire::Map, json!({"env": "prod"})),
        ] {
            let property = tags_property(wire);
            for op in [
                add(dsl_tags(&[("env", "prod")])),
                replace(dsl_tags(&[("env", "prod")])),
                reassert(dsl_tags(&[("env", "prod")])),
            ] {
                assert_eq!(
                    property.update_patch(&op),
                    Some(json!({
                        "op": "add",
                        "path": "/Tags",
                        "value": expected_value,
                    }))
                );
            }
        }
    }

    #[test]
    fn update_patch_removes_each_wire_shape_for_remove_and_empty_map() {
        for wire in [TagsWire::KeyValueList, TagsWire::Map] {
            let property = tags_property(wire);
            let expected = json!({"op": "remove", "path": "/Tags"});

            assert_eq!(property.update_patch(&remove()), Some(expected.clone()));
            assert_eq!(
                property.update_patch(&replace(dsl_tags(&[]))),
                Some(expected)
            );
        }
    }

    #[test]
    fn update_patch_rejects_unconvertible_tags() {
        let property = tags_property(TagsWire::Map);
        let invalid = Value::Concrete(ConcreteValue::Map(IndexMap::from([(
            "env".to_string(),
            Value::Concrete(ConcreteValue::Bool(true)),
        )])));

        assert_eq!(property.update_patch(&replace(invalid)), None);
    }
}
