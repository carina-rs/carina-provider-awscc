#![cfg(not(target_arch = "wasm32"))]

#[path = "support/field_groups.rs"]
mod support;

use carina_core::protocol_convert::{protocol_to_resource_schema, resource_schema_to_protocol};
use carina_core::schema::ValidatedResourceSchema;

fn protocol_round_trip(schema: &ValidatedResourceSchema) -> ValidatedResourceSchema {
    let wire = resource_schema_to_protocol(schema).unwrap();
    let json = serde_json::to_string(&wire).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    protocol_to_resource_schema(&decoded).unwrap()
}

#[test]
fn step6_cached_field_groups_validate_natively() {
    for schema in support::cached_schemas() {
        support::assert_cached_group(&schema);
    }
}

#[test]
fn step6_cached_field_groups_survive_protocol_json() {
    for schema in support::cached_schemas() {
        support::assert_cached_group(&protocol_round_trip(&schema));
    }
}

#[test]
fn step6_root_and_nested_groups_validate_natively() {
    for exactly_one in [false, true] {
        support::assert_synthetic_groups(&support::synthetic_schema(exactly_one), exactly_one);
    }
}

#[test]
fn step6_root_and_nested_groups_survive_protocol_json() {
    for exactly_one in [false, true] {
        let schema = protocol_round_trip(&support::synthetic_schema(exactly_one));
        support::assert_synthetic_groups(&schema, exactly_one);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step6_cached_field_groups_survive_loaded_wasm() {
    use carina_core::provider::ProviderFactory;
    use carina_plugin_host::WasmProviderFactory;
    use carina_provider_resolver::{ProviderLoadIdentity, ProviderTrustClass, VerifiedLoad};

    let wasm = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .ancestors()
        .map(|dir| dir.join("wasm32-wasip2/release/carina-provider-awscc.wasm"))
        .find(|path| path.exists())
        .expect("build the release wasm32-wasip2 provider before this test");
    // This regression must exercise a real guest; a missing build is a failure.
    let bytes =
        std::fs::read(wasm).expect("build the release wasm32-wasip2 provider before this test");
    let load = VerifiedLoad::for_test(
        &bytes,
        ProviderLoadIdentity::for_test("awscc", ProviderTrustClass::Untrusted),
    );
    let cache = tempfile::tempdir().unwrap();
    let factory = WasmProviderFactory::from_bytes_with_cache_dir(load, cache.path())
        .await
        .unwrap();
    let witness = factory.schema_witness();
    for native in support::cached_schemas() {
        let schema = witness
            .resources()
            .iter()
            .find(|schema| schema.parts().resource_type == native.parts().resource_type)
            .unwrap();
        support::assert_cached_group(schema);
    }
}
