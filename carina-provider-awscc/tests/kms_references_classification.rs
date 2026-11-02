//! Exercise classification directly, outside either renderer's setup.
#![cfg(not(target_arch = "wasm32"))]

#[allow(dead_code)] // The shared CFN model includes keywords used only by renderers.
#[path = "../src/bin/codegen/cfn.rs"]
mod cfn;
use cfn::*;
use serde_json::{Value, json};
#[allow(dead_code)] // Constructors and final-registry checks live in other targets.
#[path = "../src/bin/codegen/kms_references.rs"]
mod kms_references;
#[path = "../src/bin/codegen/kms_references_cfn.rs"]
mod kms_references_cfn;

#[test]
fn direct_classification_returns_an_error_instead_of_panicking() {
    let schema: CfnSchema = serde_json::from_value(serde_json::json!({
        "typeName": "AWS::EC2::VPC",
        "properties": {"KmsFoo": {"type": "string"}}
    }))
    .unwrap();
    let result =
        std::panic::catch_unwind(|| kms_references_cfn::ClassifiedSchema::new(&schema, []));
    assert!(
        result.is_ok(),
        "direct classification must return its error without a renderer guard"
    );
    let Err(error) = result.unwrap() else {
        panic!("unclassified input must return an error")
    };
    assert!(
        error
            .to_string()
            .contains("unclassified KMS string ec2.Vpc.kms_foo")
    );
}

#[test]
fn read_only_unresolved_kms_reference_does_not_require_input_classification() {
    let schema: CfnSchema = serde_json::from_value(json!({
        "typeName": "AWS::EC2::VPC",
        "properties": {"KmsOutput": {"$ref": "#/definitions/Unknown"}},
        "readOnlyProperties": ["/properties/KmsOutput"]
    }))
    .unwrap();
    assert!(kms_references_cfn::ClassifiedSchema::new(&schema, []).is_ok());
}

#[test]
fn same_named_inline_objects_keep_distinct_reference_sites() {
    let mut schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__ECS__Cluster.json"
    ))
    .unwrap();
    // Inline the classified managed-storage path. The unrelated inline object
    // has the same raw name and leaf name, but contains an ordinary string.
    let managed = schema["definitions"]["ManagedStorageConfiguration"].clone();
    schema["definitions"]["ClusterConfiguration"]["properties"]["ManagedStorageConfiguration"] =
        managed;
    schema["properties"]["Unrelated"] = json!({"type": "object", "properties": {
        "ManagedStorageConfiguration": {"type": "object", "properties": {
            "KmsKeyId": {"type": "string"}
        }}
    }});
    // This leaf is output-only, so it does not require a table decision and
    // must not inherit the writable reference at the other full path.
    schema["readOnlyProperties"]
        .as_array_mut()
        .unwrap()
        .push(json!(
            "/properties/Unrelated/ManagedStorageConfiguration/KmsKeyId"
        ));
    let schema: CfnSchema = serde_json::from_value(schema).unwrap();
    let classified = kms_references_cfn::ClassifiedSchema::new(&schema, []).unwrap();
    let reference = classified.string_leaf_at(&[
        "Configuration".into(),
        "ManagedStorageConfiguration".into(),
        "KmsKeyId".into(),
    ]);
    assert!(reference.unwrap().reference().is_some());
    let unrelated = classified.string_leaf_at(&[
        "Unrelated".into(),
        "ManagedStorageConfiguration".into(),
        "KmsKeyId".into(),
    ]);
    assert!(
        unrelated.unwrap().reference().is_none(),
        "the unrelated full path must not inherit the reference"
    );
}

#[test]
fn emission_inputs_are_bound_to_the_walked_resource_and_full_path() {
    let schema: CfnSchema = serde_json::from_value(json!({
        "typeName": "AWS::EC2::VPC",
        "properties": {"Settings": {"type": "object", "properties": {
            "Label": {"enum": ["ordinary"]}
        }}}
    }))
    .unwrap();
    let classified = kms_references_cfn::ClassifiedSchema::new(&schema, []).unwrap();
    let path = ["Settings".into(), "Label".into()];
    let leaf = classified.string_leaf_at(&path).unwrap();
    assert_eq!(leaf.cfn_type(), "AWS::EC2::VPC");
    assert_eq!(leaf.name(), "Label");
    assert_eq!(leaf.path(), path);
    assert_eq!(
        leaf.property().enum_values.as_ref().unwrap()[0].to_string_value(),
        "ordinary"
    );
    assert!(leaf.reference().is_none());
    let mut foreign = leaf.property().clone();
    foreign.description = Some("The ID of the AWS KMS key".into());
    assert!(classified.property_at(&path, &foreign).is_err());
    let property = classified.property_at(&path, leaf.property()).unwrap();
    assert_eq!(property.name(), "Label");
    assert_eq!(property.path(), path);
    assert!(std::ptr::eq(property.schema(), &classified));
    assert_eq!(property.property(), leaf.property());
    for path in [
        vec!["Label".into()],
        vec!["Settings".into(), "Unknown".into()],
    ] {
        let Err(error) = classified.string_leaf_at(&path) else {
            panic!("an unwalked leaf cannot obtain an emission input");
        };
        assert!(error.to_string().contains("unclassified string leaf path"));
    }
}

#[test]
fn scalar_collections_keep_the_exact_field_read_only_decision() {
    for property in [
        json!({"type": "array", "items": {"type": "string"}}),
        json!({"type": "object", "patternProperties": {".*": {"type": "string"}}}),
    ] {
        let schema: CfnSchema = serde_json::from_value(json!({
            "typeName": "AWS::EC2::VPC", "properties": {"KmsOutput": property},
            "readOnlyProperties": ["/properties/KmsOutput"]
        }))
        .unwrap();
        let classified = kms_references_cfn::ClassifiedSchema::new(&schema, []).unwrap();
        assert!(
            classified
                .string_leaf_at(&["KmsOutput".into(), "*".into()])
                .unwrap()
                .reference()
                .is_none()
        );
    }
}

#[test]
fn scalar_enum_discovery_stops_at_recursive_array_definitions() {
    let schema: CfnSchema = serde_json::from_value(json!({
        "typeName": "AWS::EC2::VPC",
        "properties": {"Values": {"$ref": "#/definitions/RecursiveArray"}},
        "definitions": {"RecursiveArray": {
            "type": "array", "items": {"$ref": "#/definitions/RecursiveArray"}
        }}
    }))
    .unwrap();
    assert!(!cfn::has_scalar_leaf(&schema.properties["Values"], &schema));
    assert!(kms_references_cfn::ClassifiedSchema::new(&schema, []).is_ok());
}

#[test]
fn provider_populated_tag_projection_has_no_writable_source_leaves() {
    let schema: CfnSchema = serde_json::from_value(json!({
        "typeName": "AWS::EC2::VPC",
        "properties": {"Tags": {"type": "array", "items": {"$ref": "#/definitions/Tag"}}},
        "definitions": {"Tag": {"type": "object", "properties": {
            "Key": {"type": "string"},
            "Value": {"type": "string", "description": "The AWS KMS key"}
        }}},
        "tagging": {"taggable": true, "tagProperty": "/properties/Tags"},
        "readOnlyProperties": ["/properties/Tags"]
    }))
    .unwrap();
    assert!(kms_references_cfn::ClassifiedSchema::new(&schema, []).is_ok());
}

#[test]
fn replaced_properties_audit_subtrees_supplied_by_resource_branches() {
    for keyword in ["oneOf", "anyOf", "allOf"] {
        let mut value = json!({
            "typeName": "AWS::EC2::VPC",
            "properties": {"Settings": {"type": "object"}}
        });
        value[keyword] = json!([{"properties": {"Settings": {"properties": {
            "Value": {"type": "string", "description": "The AWS KMS key"}
        }}}}]);
        let schema: CfnSchema = serde_json::from_value(value).unwrap();
        let Err(error) =
            kms_references_cfn::ClassifiedSchema::new(&schema, [&schema.properties["Settings"]])
        else {
            panic!("{keyword} hid a CFN input below a replacement type");
        };
        assert!(
            error
                .to_string()
                .contains("unclassified KMS string ec2.Vpc.settings.value"),
            "{error}"
        );
    }
}
