use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity_inventory::Inventory;
use carina_core::parser::{LoadedProviderContext, ProviderContext, parse_file};
use carina_core::provider::{ProviderFactory, provider_custom_types_from_factories};
use carina_core::schema::{AttributeSchema, ResourceSchema};

fn native_inventory() -> Inventory {
    Inventory::schemas(
        &super::all_schemas(),
        &crate::AwsccProviderFactory.provider_config_attribute_types(),
    )
}

fn catalog_inventory() -> Inventory {
    let mut catalog = Inventory::default();
    for ty in super::catalog::AwsccType::ALL {
        catalog.identities.insert(ty.identity());
        catalog.walk(&ty.attribute_type(), &BTreeMap::new());
    }
    for ty in super::string_identity::StringIdentity::ALL {
        catalog.identities.insert(ty.identity());
        catalog.walk(&ty.declaration(), &BTreeMap::new());
    }
    for ty in super::catalog::ScalarType::ALL {
        catalog.identities.insert(ty.identity());
        catalog.walk(&ty.attribute_type(), &BTreeMap::new());
    }
    catalog
}

#[test]
fn namespace_walk_every_native_identity_is_awscc_or_builtin() {
    // Clippy cannot ban TypeIdentity struct literals. This schema walk is the
    // backstop for identities that bypass the construction seam that way.
    native_inventory().assert_owned();
    catalog_inventory().assert_owned();
}

#[test]
#[should_panic(expected = "AWSCC declares foreign identities")]
fn namespace_walk_rejects_providerless_nonbuiltin_identity() {
    // A lost provider must not disguise an AWSCC declaration as a core builtin.
    let declaration = carina_core::schema::AttributeType::refined_string(
        Some(carina_core::schema::TypeIdentity {
            provider: None,
            segments: vec!["ec2".into(), "Vpc".into()],
            kind: "Id".into(),
        }),
        None,
        None,
        None,
    );
    let mut inventory = Inventory::default();
    inventory.walk(&declaration, &BTreeMap::new());
    inventory.assert_owned();
}

#[test]
fn namespace_walk_accepts_core_builtin_registry() {
    let builtins = carina_core::schema::builtin_type_registry();
    let mut inventory = Inventory::default();
    for (identity, declaration) in builtins {
        assert_eq!(identity.provider, None);
        inventory.walk(declaration, &BTreeMap::new());
    }
    assert_eq!(inventory.identities, builtins.keys().cloned().collect());
    inventory.assert_owned();
}

#[test]
fn namespace_old_full_form_enums_are_rejected_by_host() {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
    let mut registry = super::schema_registry();
    let mut inventory = native_inventory();
    inventory.enums.extend(catalog_inventory().enums);
    for (segments, kind) in [
        (vec![], "Region"),
        (vec!["AvailabilityZone"], "ZoneName"),
        (vec!["iam", "PolicyDocument"], "Version"),
        (vec!["iam", "PolicyDocument", "Statement"], "Effect"),
        (
            vec!["iam", "PolicyDocument", "Statement", "Condition"],
            "ConditionOperator",
        ),
    ] {
        assert!(
            inventory
                .enums
                .keys()
                .any(|id| id.kind == kind && id.segments == segments),
            "missing required enum probe {segments:?}.{kind}"
        );
    }
    let mut probes = Vec::new();
    for (i, (identity, ty)) in inventory.enums.iter().enumerate() {
        assert!(identity.provider.is_some(), "namespaced enum {identity}");
        let (_, values, aliases, _, _) = ty.enum_parts().unwrap();
        let member = aliases
            .first()
            .map(|(_, dsl)| dsl.as_str())
            .or_else(|| values.and_then(|values| values.first()).map(String::as_str))
            .unwrap_or_else(|| match identity.kind.as_str() {
                "Region" => "ap_northeast_1",
                "ZoneName" => "ap_northeast_1a",
                kind => panic!("open enum {kind} needs a valid probe"),
            });
        let tail = identity
            .segments
            .iter()
            .chain(std::iter::once(&identity.kind))
            .cloned()
            .collect::<Vec<_>>()
            .join(".");
        let resource = format!("namespace.Probe{i}");
        let schema = ResourceSchema::new(&resource)
            .attribute(AttributeSchema::required("value", ty.clone()).build())
            .validate()
            .unwrap();
        registry.insert("awscc", schema);
        probes.push((
            resource,
            format!("awscc.{tail}.{member}"),
            format!("aws.{tail}.{member}"),
        ));
    }
    let loaded = LoadedProviderContext::for_test(ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&registry),
        ..ProviderContext::new(carina_core::EnvironmentSnapshot::empty())
    });
    assert!(
        probes.len() > 247,
        "generated, handwritten, PolicyDocument and Region/AZ enums must be covered"
    );
    for (resource, new, old) in probes {
        let source = |value: &str| format!("awscc.{resource} {{ value = {value} }}");
        let validate = |value: &str| {
            let mut parsed = parse_file(
                &source(value),
                std::path::Path::new("namespace.crn"),
                loaded.as_provider_context(),
            )
            .map_err(|e| e.to_string())?;
            carina_core::schema::resolve_file_block_names_with_schemas(&mut parsed, &registry)
                .map_err(|errors| format!("{errors:?}"))?;
            carina_core::identifier::resolve_attr_prefixes(&mut parsed.resources, &registry)
                .map_err(|error| error.to_string())?;
            carina_core::validation::validate_resources(
                &parsed,
                &registry,
                &std::collections::HashSet::from(["awscc".into()]),
                loaded.as_provider_context(),
                &mut Vec::new(),
            )
            .map_err(|e| e.to_string())
        };
        validate(&new).unwrap_or_else(|e| panic!("{new}: {e}"));
        assert!(
            validate(&old).is_err(),
            "old full form is still accepted: {old}"
        );
    }
}

#[test]
fn namespace_provider_region_rejects_old_full_form_through_host() {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
    let loaded = LoadedProviderContext::for_test(ProviderContext {
        custom_types: provider_custom_types_from_factories(factories.clone()),
        resource_types: ProviderContext::resource_types_from_schema_registry(
            &super::schema_registry(),
        ),
        ..ProviderContext::new(carina_core::EnvironmentSnapshot::empty())
    });
    for (region, accepted) in [
        ("awscc.Region.ap_northeast_1", true),
        ("aws.Region.ap_northeast_1", false),
    ] {
        let source = format!("provider awscc {{ region = {region} }}");
        let result = parse_file(
            &source,
            std::path::Path::new("provider.crn"),
            loaded.as_provider_context(),
        )
        .map_err(|error| error.to_string())
        .and_then(|parsed| carina_core::validation::validate_provider_config(&parsed, &factories));
        assert_eq!(result.is_ok(), accepted, "{region}: {result:?}");
    }
}

#[test]
#[cfg(unix)]
fn namespace_generation_aborts_if_codegen_build_fails() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = tempfile::tempdir().unwrap();
    let bin = scratch.path().join("bin");
    let target = scratch.path().join("target");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(target.join("debug")).unwrap();
    for (path, source) in [
        (bin.join("cargo"), "#!/bin/sh\nexit 42\n"),
        (
            target.join("debug/codegen"),
            "#!/bin/sh\necho stale-codegen-executed\nexit 99\n",
        ),
    ] {
        std::fs::write(&path, source).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = std::process::Command::new("bash")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/generate-schemas.sh"
        ))
        .arg("--from-cache-only")
        .current_dir(scratch.path())
        // Fake cargo compiles nothing; inherit no build environment from nextest.
        .env_clear()
        .env(
            "PATH",
            format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
        )
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(42),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("stale-codegen-executed"));
}
