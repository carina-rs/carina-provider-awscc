use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use carina_core::binding_index::{LivePreApplyInputs, ResolvedBindings};
use carina_core::config_loader::load_buffer_configuration;
use carina_core::data_source_value::DataSourceValues;
use carina_core::parser::{
    LoadedProviderContext, ProviderContext, RelativeBindingName, parse_file,
};
use carina_core::provider::{ProviderFactory, provider_custom_types_from_factories};
use carina_core::resolver::resolve_ref_value_with_config;
use carina_core::resource::{AuthoredConcreteValue, AuthoredResource, AuthoredValue};
use carina_core::schema::{
    RawShape, SchemaKind, StringIdentityParts, StringMemberParts, TypeAssignability, TypeIdentity,
    ValidatedResourceSchema, builtin_type_registry,
};
use carina_core::validation::{
    deferred_types::with_session_for_test, typecheck::check_type_expr_value,
};

use crate::test_support::{
    AssignmentContext, field, int, list, string, try_validate_fixtures, validate_fixtures,
};

fn role_schema() -> ValidatedResourceSchema {
    super::generated::iam::role::iam_role_config().schema
}

fn role_policy_schema() -> ValidatedResourceSchema {
    super::generated::iam::role_policy::iam_role_policy_config().schema
}

fn oidc_schema() -> ValidatedResourceSchema {
    super::generated::iam::oidc_provider::iam_oidc_provider_config().schema
}

#[test]
fn role_and_policy_names_use_shared_identities_and_documented_boundaries() {
    let role = role_schema();
    let policy = role_policy_schema();
    for (schema, path, shared, maximum) in [
        (
            &role,
            "role_name",
            carina_aws_types::AwsType::IamRoleName,
            64,
        ),
        (
            &policy,
            "role_name",
            carina_aws_types::AwsType::IamRoleName,
            64,
        ),
        (
            &policy,
            "policy_name",
            carina_aws_types::AwsType::IamPolicyName,
            128,
        ),
        (
            &role,
            "policies.policy_name",
            carina_aws_types::AwsType::IamPolicyName,
            128,
        ),
    ] {
        let name = field(schema, path);
        assert!(matches!(
            name.as_type().raw_shape(),
            RawShape::String { identity_layer: StringIdentityParts { identity: Some(identity), length: Some((Some(1), Some(max))), .. }, .. }
                if identity == &shared.identity() && max == maximum
        ));
        for value in [
            "a".to_string(),
            "AZaz09_+=,.@-".to_string(),
            "a".repeat(maximum as usize),
        ] {
            assert!(name.validate(&string(&value)).is_ok(), "{path}: {value:?}");
        }
        for value in [
            "".to_string(),
            "a".repeat(maximum as usize + 1),
            "bad name".to_string(),
            "name/".to_string(),
            "rôle".to_string(),
            "valid\n".to_string(),
        ] {
            assert!(name.validate(&string(&value)).is_err(), "{path}: {value:?}");
        }
        assert!(name.validate(&int(1)).is_err(), "{path} must remain String");
    }
}

#[test]
fn role_path_uses_shared_anonymous_path_type_and_keeps_the_root_path() {
    let schema = role_schema();
    let path = field(&schema, "path");
    let shared = carina_aws_types::iam_path();
    let RawShape::String {
        identity_layer:
            StringIdentityParts {
                identity,
                pattern,
                length,
                ..
            },
        ..
    } = path.as_type().raw_shape()
    else {
        panic!("IAM path must remain String");
    };
    let RawShape::String {
        identity_layer:
            StringIdentityParts {
                identity: None,
                pattern: None,
                length: None,
                ..
            },
        member_layer:
            StringMemberParts {
                pattern: shared_pattern,
                length: shared_length,
            },
    } = shared.raw_shape()
    else {
        panic!("shared IAM path must keep anonymous constraints in its member layer");
    };
    assert!(identity.is_none());
    assert_eq!(pattern, shared_pattern);
    assert_eq!(length, shared_length);
    for value in [
        "/".to_string(),
        "/carina/app/".to_string(),
        "/!~+=,.@-_/".to_string(),
        format!("/{}/", "a".repeat(510)),
    ] {
        assert!(path.validate(&string(&value)).is_ok(), "{value:?}");
    }
    for value in [
        "".to_string(),
        "carina/".to_string(),
        "/carina".to_string(),
        "/bad path/".to_string(),
        "/é/".to_string(),
        "/\u{7f}/".to_string(),
        "/valid/\n".to_string(),
        format!("/{}/", "a".repeat(511)),
    ] {
        assert!(path.validate(&string(&value)).is_err(), "{value:?}");
    }
}

#[test]
fn role_description_accepts_only_the_create_role_character_set_and_length() {
    let schema = role_schema();
    let description = field(&schema, "description");
    assert!(matches!(
        description.as_type().raw_shape(),
        RawShape::String {
            identity_layer: StringIdentityParts {
                identity: None,
                length: Some((None, Some(1000))),
                ..
            },
            ..
        }
    ));
    for value in [
        "".to_string(),
        "\t\n\r !~¡ÿ".to_string(),
        "a".repeat(1000),
        "ÿ".repeat(1000),
    ] {
        assert!(description.validate(&string(&value)).is_ok(), "{value:?}");
    }
    for value in [
        "a".repeat(1001),
        "ÿ".repeat(1001),
        "\0".to_string(),
        "\u{000b}".to_string(),
        "\u{001f}".to_string(),
        "\u{007f}".to_string(),
        "\u{0085}".to_string(),
        "\u{00a0}".to_string(),
        "\u{0100}".to_string(),
    ] {
        assert!(description.validate(&string(&value)).is_err(), "{value:?}");
    }
    assert!(description.validate(&int(1)).is_err());
}

#[test]
fn oidc_url_reuses_core_https_url_through_the_provider_registry() {
    // Individual resource admission did not catch the built-in collision.
    let registry = super::schema_registry();
    let schema = registry
        .get("awscc", "iam.OidcProvider", SchemaKind::Resource)
        .expect("OIDC provider must be present in the aggregate registry");
    let url = field(schema.schema(), "url");
    let RawShape::String {
        identity_layer:
            StringIdentityParts {
                identity: Some(identity),
                pattern: None,
                length: None,
                validate: Some(validate),
                ..
            },
        member_layer:
            carina_core::schema::StringMemberParts {
                pattern: None,
                length: Some((Some(1), Some(255))),
            },
    } = url.as_type().raw_shape()
    else {
        panic!("OIDC URL must preserve core HttpsUrl with CFN member length");
    };
    assert_eq!(identity, &TypeIdentity::bare("HttpsUrl"));
    let builtin = builtin_type_registry().get(identity).unwrap();
    let RawShape::String {
        identity_layer:
            StringIdentityParts {
                validate: Some(builtin_validate),
                ..
            },
        ..
    } = builtin.raw_shape()
    else {
        panic!("core HttpsUrl must have a validator");
    };
    assert!(Arc::ptr_eq(validate, builtin_validate));
    let prefix = "https://example.com/";
    for value in [
        "https://example.com".to_string(),
        "https://example.com/oidc/issuer".to_string(),
        "https://example.com/%3Fescaped".to_string(),
        "https://example.com/#fragment?text".to_string(),
        // The API says ports "should not" be used; no mandatory port ban.
        "https://example.com:8443/issuer".to_string(),
        format!("{prefix}{}", "a".repeat(255 - prefix.len())),
        // Preserve HttpsUrl's grammar, including query components. Only the
        // CFN length is a member restriction on this slot.
        "https://example.com/?aud=client".to_string(),
        "https://example.com/?aud=client#fragment".to_string(),
    ] {
        assert!(url.validate(&string(&value)).is_ok(), "{value:?}");
    }
    let too_long = format!("{prefix}{}", "a".repeat(256 - prefix.len()));
    assert!(builtin_validate(&string(&too_long)).is_ok());
    for value in [
        too_long,
        "".to_string(),
        "x".to_string(),
        "http://example.com".to_string(),
        "https://".to_string(),
        "https://user@example.com".to_string(),
        "https://[invalid]".to_string(),
        "https://example.com:0".to_string(),
        "https://example.com/\n".to_string(),
    ] {
        assert!(url.validate(&string(&value)).is_err(), "{value:?}");
    }
    assert!(url.validate(&int(1)).is_err());
}

#[test]
fn oidc_client_ids_enforce_the_collection_limit_and_existing_item_lengths() {
    let schema = oidc_schema();
    let ids = field(&schema, "client_id_list");
    assert!(matches!(
        ids.as_type().raw_shape(),
        RawShape::List { ordered: false, .. }
    ));
    for count in [0, 1, 100] {
        ids.validate(&list(
            (0..count).map(|i| string(&format!("client-{i}"))).collect(),
        ))
        .unwrap();
    }
    assert!(
        ids.validate(&list(
            (0..101).map(|i| string(&format!("client-{i}"))).collect()
        ))
        .is_err()
    );
    for length in [1, 255] {
        ids.validate(&list(vec![string(&"a".repeat(length))]))
            .unwrap();
    }
    for length in [0, 256] {
        assert!(
            ids.validate(&list(vec![string(&"a".repeat(length))]))
                .is_err()
        );
    }
    assert!(ids.validate(&list(vec![int(1)])).is_err());
}

#[test]
fn existing_session_duration_and_thumbprint_bounds_are_preserved() {
    let role = role_schema();
    let duration = field(&role, "max_session_duration");
    for value in [3600, 43200] {
        duration.validate(&int(value)).unwrap();
    }
    for value in [3599, 43201] {
        assert!(duration.validate(&int(value)).is_err());
    }
    let oidc = oidc_schema();
    let thumbprints = field(&oidc, "thumbprint_list");
    let thumbprint = "0123456789abcdef0123456789ABCDEF01234567";
    for count in [0, 1, 5] {
        thumbprints
            .validate(&list(vec![string(thumbprint); count]))
            .unwrap();
    }
    assert!(
        thumbprints
            .validate(&list(vec![string(thumbprint); 6]))
            .is_err()
    );
    for value in ["a".repeat(39), "a".repeat(41), "g".repeat(40)] {
        assert!(thumbprints.validate(&list(vec![string(&value)])).is_err());
    }
}

// The four sinks exercise Role's producer, both RolePolicy inputs, and the
// nested inline policy. Keep literal values valid so a failure identifies the
// reference boundary rather than an unrelated policy-document constraint.
fn name_fixture(declarations: &str, names: [&str; 4]) -> String {
    format!(
        r#"
provider awscc {{
  region = awscc.Region.ap_northeast_1
}}
{}"#,
        module_name_fixture(declarations, names)
    )
}

fn module_name_fixture(declarations: &str, names: [&str; 4]) -> String {
    let [
        role_name,
        attached_role_name,
        policy_name,
        inline_policy_name,
    ] = names;
    format!(
        r#"
{declarations}
let role = awscc.iam.Role {{
  role_name = {role_name}
  assume_role_policy_document = {{
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {{
      effect = allow
      principal = {{ service = 'lambda.amazonaws.com' }}
      action = 'sts:AssumeRole'
    }}
  }}
  policy {{
    policy_name = {inline_policy_name}
    policy_document = {{
      version = aws.iam.PolicyDocument.Version.2012_10_17
      statement {{ effect = allow action = 's3:ListAllMyBuckets' resource = '*' }}
    }}
  }}
}}
awscc.iam.RolePolicy {{
  role_name = {attached_role_name}
  policy_name = {policy_name}
}}
"#
    )
}

const LITERAL_NAMES: [&str; 4] = [
    "'RoleName'",
    "'RoleName'",
    "'PolicyName'",
    "'InlinePolicyName'",
];
const NAME_IDENTITIES: [&str; 4] = [
    "aws.iam.Role.Name",
    "aws.iam.Role.Name",
    "aws.iam.Policy.Name",
    "aws.iam.Policy.Name",
];
const PLAIN_STRING_OUTPUT: &str = r#"
let source = awscc.ssm.Parameter {
  name = '/carina/iam-name'
  type = string
  value = 'ValidName'
}
let x = source.value
"#;

#[test]
fn role_name_output_is_assignable_to_role_policy_and_the_full_fixture_validates() {
    let assignment = AssignmentContext::new();
    let role = role_schema();
    let policy = role_policy_schema();
    assert!(
        role.as_schema()
            .type_in_schema(field(&role, "role_name").as_type())
            .is_assignable_to(assignment.sink(&policy, field(&policy, "role_name").as_type()))
    );
    let mut names = LITERAL_NAMES;
    names[1] = "role.role_name";
    validate_fixtures(&[("iam_role_name_reference.crn", &name_fixture("", names))]);
}

#[test]
fn loaded_catalog_rejects_role_name_as_policy_name() {
    let assignment = AssignmentContext::new();
    let role = role_schema();
    let policy = role_policy_schema();
    let result = role
        .as_schema()
        .type_in_schema(field(&role, "role_name").as_type())
        .assignment_to(assignment.sink(&policy, field(&policy, "policy_name").as_type()));

    assert_eq!(result, TypeAssignability::Mismatch);
    assert!(matches!(result.require_loaded(), Ok(false)));
}

#[test]
fn literal_lets_validate_at_each_name_sink() {
    for site in 0..LITERAL_NAMES.len() {
        let mut names = LITERAL_NAMES;
        names[site] = "x";
        validate_fixtures(&[(
            "iam_literal_let.crn",
            &name_fixture("let x = 'ValidName'", names),
        )]);
    }
}

#[test]
fn named_module_arguments_validate_at_each_name_sink() {
    for (site, identity) in NAME_IDENTITIES.iter().enumerate() {
        let mut names = LITERAL_NAMES;
        names[site] = "x";
        validate_fixtures(&[(
            "iam_named_argument.crn",
            &module_name_fixture(
                &format!("arguments {{ x: {identity} = 'ValidName' }}"),
                names,
            ),
        )]);
    }
}

#[test]
fn named_iam_names_reject_plain_string_references_even_with_valid_literal_defaults() {
    let parameter = super::generated::ssm::parameter::ssm_parameter_config().schema;
    assert!(matches!(
        field(&parameter, "value").as_type().raw_shape(),
        RawShape::String {
            identity_layer: StringIdentityParts {
                identity: None,
                pattern: None,
                length: None,
                ..
            },
            ..
        }
    ));
    for (site, identity) in NAME_IDENTITIES.iter().enumerate() {
        let mut names = LITERAL_NAMES;
        names[site] = "x";
        for fixture in [
            name_fixture(PLAIN_STRING_OUTPUT, names),
            module_name_fixture("arguments { x: String = 'ValidName' }", names),
        ] {
            let error = try_validate_fixtures(&[("iam_plain_string_reference.crn", &fixture)])
                .expect_err("a declared String reference does not prove the named identity");
            assert!(
                error.contains(&format!("expected {identity}, got String")),
                "site {site}: {error}"
            );
        }
    }
}

#[test]
fn direct_interpolations_validate_and_defer_name_constraints_until_the_value_resolves() {
    // A string construction retains its interpolation judgement and defers
    // destination constraints. Its operands need not carry the named identity.
    for site in 0..LITERAL_NAMES.len() {
        let mut names = LITERAL_NAMES;
        names[site] = r#""${x}-inline""#;
        for fixture in [
            name_fixture(PLAIN_STRING_OUTPUT, names),
            module_name_fixture("arguments { x: String = 'ValidName' }", names),
        ] {
            validate_fixtures(&[("iam_interpolated_name.crn", &fixture)]);
        }
    }
    // Fully concrete interpolation folds to a literal; validate its value now.
    for site in 0..LITERAL_NAMES.len() {
        for interpolation in [r#""${x}""#, r#""${x}-inline""#] {
            let mut names = LITERAL_NAMES;
            names[site] = interpolation;
            validate_fixtures(&[(
                "iam_constant_name.crn",
                &name_fixture("let x = 'ValidName'", names),
            )]);
            assert!(
                try_validate_fixtures(&[(
                    "iam_invalid_constant_name.crn",
                    &name_fixture("let x = 'bad name'", names)
                )])
                .is_err(),
                "site {site}, {interpolation}: invalid concrete names must be rejected"
            );
        }
    }
}

#[test]
fn bare_interpolations_preserve_reference_types_at_each_name_sink() {
    // A bare interpolation preserves its operand's declared type, including
    // through a let. String constructions instead defer the value constraints.
    for (site, identity) in NAME_IDENTITIES.iter().enumerate() {
        for (rhs, alias) in [
            (r#""${x}""#, ""),
            ("interpolated", r#"let interpolated = "${x}""#),
        ] {
            let mut names = LITERAL_NAMES;
            names[site] = rhs;
            for fixture in [
                name_fixture(&format!("{PLAIN_STRING_OUTPUT}\n{alias}"), names),
                module_name_fixture(
                    &format!("arguments {{ x: String = 'ValidName' }}\n{alias}"),
                    names,
                ),
            ] {
                let error = try_validate_fixtures(&[("iam_bare_interpolation.crn", &fixture)])
                    .expect_err("a bare interpolation cannot turn String into a named IAM type");
                assert!(
                    error.contains(&format!("expected {identity}, got String")),
                    "site {site}, {rhs}: {error}"
                );
            }
            validate_fixtures(&[(
                "iam_named_bare_interpolation.crn",
                &module_name_fixture(
                    &format!("arguments {{ x: {identity} = 'ValidName' }}\n{alias}"),
                    names,
                ),
            )]);
        }
    }
}

#[test]
fn deferred_interpolated_role_names_validate_constraints_after_resolution() {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
    let schemas = super::schema_registry();
    let context = LoadedProviderContext::for_test(ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::new(carina_core::EnvironmentSnapshot::empty())
    });
    for (interpolation, suffix) in [(r#""${x}""#, ""), (r#""${x}-inline""#, "-inline")] {
        let fixture = format!(
            "{}\nlet x = role.role_name\nlet interpolated = {interpolation}",
            name_fixture("", LITERAL_NAMES)
        );
        let parsed = parse_file(
            &fixture,
            Path::new("iam_resolved_interpolation.crn"),
            &context,
        )
        .unwrap();
        let configuration = load_buffer_configuration(&parsed, &schemas, &context);
        let report = configuration.judged().report();
        // Derive the exact destination declaration from the registered IAM field.
        let expected = report
            .binding(&RelativeBindingName::new("x"))
            .unwrap()
            .static_type()
            .unwrap();
        assert_eq!(expected.to_string(), NAME_IDENTITIES[0]);
        let expression = report
            .binding(&RelativeBindingName::new("interpolated"))
            .unwrap();
        let authored = parsed.variables["interpolated"]
            .clone()
            .resolve_root_for_test()
            .into_expanded()
            .unwrap();
        assert!(
            authored
                .knowledge(&carina_core::resource::knowledge::KnowledgeContext::validation())
                .unwrap()
                .into_classification()
                .into_known()
                .is_err()
        );
        assert!(
            check_type_expr_value(expected, &authored, &context, expression)
                .require_provider_schemas()
                .is_ok()
        );

        // Match core's resolution-time constraint test: resolve pre-apply
        // bindings, then check the known value with the retained judgement.
        for (value, accepted) in [
            ("ValidName".to_string(), true),
            ("a".repeat(64 - suffix.len()), true),
            ("a".repeat(65 - suffix.len()), false),
            ("bad name".to_string(), false),
        ] {
            let source = AuthoredResource::with_provider("awscc", "iam.Role", "role", None)
                .with_binding("role")
                .with_attribute(
                    "role_name",
                    AuthoredValue::Concrete(AuthoredConcreteValue::String(value.clone().into())),
                )
                .resolve_root_for_test()
                .into_expanded(&schemas)
                .unwrap();
            let bindings = ResolvedBindings::pre_apply_live(LivePreApplyInputs {
                managed: &[source],
                compositions: &[],
                managed_current: &HashMap::new(),
                data_source_values: &DataSourceValues::empty(),
                remote_bindings: &HashMap::new(),
                wait_aliases: &[],
            });
            let resolved = with_session_for_test(&context, |session| {
                resolve_ref_value_with_config(&authored, &bindings, session)
            })
            .unwrap();
            assert_eq!(resolved, string(&format!("{value}{suffix}")));
            let checked = check_type_expr_value(expected, &resolved, &context, expression)
                .require_provider_schemas();
            assert_eq!(
                checked.is_ok(),
                accepted,
                "{interpolation}, {value:?}: {checked:?}"
            );
            if let Err(error) = checked {
                assert!(
                    error.to_string().contains(NAME_IDENTITIES[0]),
                    "{interpolation}, {value:?}: {error}"
                );
            }
        }
        assert!(
            authored
                .knowledge(&carina_core::resource::knowledge::KnowledgeContext::validation())
                .unwrap()
                .into_classification()
                .into_known()
                .is_err()
        );
    }
}

#[test]
fn let_bound_deferred_string_constructions_validate_at_each_name_sink() {
    // The retained judgement follows string constructions through let bindings;
    // their eventual values face the same constraints as inline constructions.
    let alias = r#"let interpolated = "${x}-inline""#;
    for site in 0..LITERAL_NAMES.len() {
        let mut names = LITERAL_NAMES;
        names[site] = "interpolated";
        for fixture in [
            name_fixture(&format!("{PLAIN_STRING_OUTPUT}\n{alias}"), names),
            module_name_fixture(
                &format!("arguments {{ x: String = 'ValidName' }}\n{alias}"),
                names,
            ),
        ] {
            validate_fixtures(&[("iam_let_interpolation.crn", &fixture)]);
        }
    }
}

#[test]
fn every_example_and_acceptance_fixture_declaring_iam_resources_keep_cfn_reference_limits() {
    fn collect(directory: &Path, root: &Path, fixtures: &mut Vec<(String, String)>) {
        let mut entries = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                collect(&path, root, fixtures);
            } else if path.extension().is_some_and(|extension| extension == "crn") {
                let source = std::fs::read_to_string(&path).unwrap();
                if source.contains("awscc.iam.") {
                    fixtures.push((
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        source,
                    ));
                }
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut fixtures = Vec::new();
    for directory in ["examples", "acceptance-tests"] {
        collect(&root.join(directory), root, &mut fixtures);
    }
    assert!(
        !fixtures.is_empty(),
        "the IAM fixture inventory must not be empty"
    );
    let cases = fixtures
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    crate::test_support::validate_fixtures(&cases);
}
