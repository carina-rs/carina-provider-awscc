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
    let result = std::panic::catch_unwind(|| kms_references_cfn::ClassifiedSchema::new(&schema));
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
    assert!(kms_references_cfn::ClassifiedSchema::new(&schema).is_ok());
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
    let classified = kms_references_cfn::ClassifiedSchema::new(&schema).unwrap();
    let reference = classified.reference_at_property(&[
        "Configuration".into(),
        "ManagedStorageConfiguration".into(),
        "KmsKeyId".into(),
    ]);
    assert!(reference.is_some());
    let unrelated = classified.reference_at_property(&[
        "Unrelated".into(),
        "ManagedStorageConfiguration".into(),
        "KmsKeyId".into(),
    ]);
    assert!(
        unrelated.is_none(),
        "the unrelated full path must not inherit the reference"
    );
}
