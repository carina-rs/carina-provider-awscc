use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use carina_core::provider::{
    ProviderFactory, collect_schemas, provider_custom_types_from_factories,
};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::SchemaKind;
use carina_plugin_host::WasmProviderFactory;
use carina_provider_resolver::{ProviderLoadIdentity, ProviderTrustClass, VerifiedLoad};

fn release_wasm_path() -> Option<PathBuf> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ["carina-provider-awscc.wasm", "carina_provider_awscc.wasm"] {
        let path = workspace_root
            .join("target/wasm32-wasip2/release")
            .join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_log_integer_validator_dispatches_through_the_wasm_guest() {
    let Some(wasm_path) = release_wasm_path() else {
        eprintln!(
            "SKIP: release WASM component not found; build with: \
             cargo build --offline -p carina-provider-awscc --target wasm32-wasip2 --release"
        );
        return;
    };

    let bytes = std::fs::read(&wasm_path).expect("read AWSCC WASM component");
    let identity = ProviderLoadIdentity::for_test("awscc", ProviderTrustClass::Untrusted);
    let load = VerifiedLoad::for_test(&bytes, identity);
    let cache = tempfile::tempdir().expect("create isolated compiled-WASM cache");
    let factory = WasmProviderFactory::from_bytes_with_cache_dir(load, cache.path())
        .await
        .expect("load AWSCC WASM component through the production host");
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> = Arc::new(vec![Box::new(factory)]);
    let schemas = collect_schemas(&factories);
    let flow_log = schemas
        .get("awscc", "ec2.FlowLog", SchemaKind::Resource)
        .expect("ec2.FlowLog schema must cross the WASM boundary");
    let custom_types = provider_custom_types_from_factories(Arc::clone(&factories));
    let validate = |value| {
        flow_log.validate_with_resolver(
            &HashMap::from([
                (
                    "max_aggregation_interval".to_string(),
                    Value::Concrete(ConcreteValue::Int(value)),
                ),
                (
                    "resource_id".to_string(),
                    Value::Concrete(ConcreteValue::String("vpc-0123456789abcdef0".into())),
                ),
                (
                    "resource_type".to_string(),
                    Value::Concrete(ConcreteValue::enum_identifier(
                        "aws.ec2.FlowLog.ResourceType.vpc",
                    )),
                ),
            ]),
            &|_| false,
            &custom_types,
        )
    };

    validate(60).expect("guest bridge must parse and accept integer witness 60");
    let errors = validate(61).expect_err(
        "guest bridge must parse and reject integer witness 61 before any operation call",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.to_string().contains("Value 61 is not a valid value")),
        "unexpected validation errors: {errors:#?}"
    );
}
