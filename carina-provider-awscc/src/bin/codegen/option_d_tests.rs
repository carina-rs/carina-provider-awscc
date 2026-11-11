use super::*;
use serde_json::json;

#[test]
fn option_d_evidence_requires_an_aws_url_quote_and_valid_date() {
    use overrides::CheckedOn;
    let date = CheckedOn { year: 2026, month: 11, day: 7 };
    let url = "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html";
    let quote = "Length Constraints: Minimum length of 1. Maximum length of 64.";
    assert!(AwsEvidence::new("", quote, date).is_err());
    assert!(AwsEvidence::new("https://example.com/", quote, date).is_err());
    assert!(AwsEvidence::new(url, " \n", date).is_err());
    assert!(AwsEvidence::new(url, quote, CheckedOn { day: 0, ..date }).is_err());
    assert!(AwsEvidence::new(url, quote, CheckedOn { month: 2, day: 30, ..date }).is_err());
    assert!(AwsEvidence::new(url, quote, date).is_ok());
}

#[test]
fn option_d_observations_require_exact_inputs_dates_directions_and_source_records() {
    use overrides::{CheckedOn, ObservedInput, VerifiedDirection};
    static INPUT: LazyLock<serde_json::Value> = LazyLock::new(|| json!({"StartDate":"2030-01-01T00:00:00.123Z"}));
    static OUTPUT: LazyLock<serde_json::Value> = LazyLock::new(|| json!({"StartDate":"2030-01-01T00:00:00.123Z"}));
    let date = CheckedOn { year: 2026, month: 11, day: 5 };
    let record = "https://github.com/carina-rs/carina-provider-awscc/pull/592";
    let cases = || vec![ObservedInput { exact_input: &INPUT, direction: VerifiedDirection::ReadBackForm, result: &OUTPUT }];
    assert!(AwsEvidence::observed(Vec::new(), "ap-northeast-1", date, record, "Create/Get").is_err());
    assert!(AwsEvidence::observed(cases(), "", date, record, "Create/Get").is_err());
    assert!(AwsEvidence::observed(cases(), "ap-northeast-1", CheckedOn { day: 0, ..date }, record, "Create/Get").is_err());
    assert!(AwsEvidence::observed(cases(), "ap-northeast-1", date, "https://github.com/carina-rs", "Create/Get").is_err());
    assert!(AwsEvidence::observed(cases(), "ap-northeast-1", date, record, "Create/Get").is_ok());
}

#[test]
fn option_d_every_reviewed_narrowing_has_complete_documentation() {
    let mut unique = BTreeSet::new();
    for row in narrowing_evidence::rows() {
        let resource = row["resource"].as_str().unwrap();
        let field = row["field"].as_str().unwrap();
        assert!(unique.insert((resource, field)), "duplicate review: {row}");
        let evidence = narrowing_evidence::for_field(resource, field);
        if row["status"] == "dropped" {
            assert!(evidence.is_none(), "{row}");
            assert!(!row["reason"].as_str().unwrap().is_empty());
        } else {
            let evidence = evidence.unwrap();
            let date = evidence.checked_on();
            if let AwsEvidence::Documentation(_) = evidence {
                assert_eq!((date.year, date.month, date.day), (2026, 11, 7));
            } else {
                assert_eq!((date.year, date.month), (2026, 11));
                assert!([5,6].contains(&date.day));
            }
            for (url, quote) in evidence.sources() {
                assert!(url.starts_with("https://docs.aws.amazon.com/"), "{row}");
                assert!(!quote.trim().is_empty(), "{row}");
            }
        }
    }
    assert_eq!(unique.len(), 349 + 7 + EMPTY_OBJECT_PROPERTIES.len());
    for ((resource, path), entry) in resource_type_overrides().iter() {
        if let Some(narrowing) = &entry.narrowing {
            assert_eq!(narrowing.evidence, narrowing_evidence::for_field(resource, path).unwrap());
        }
    }
}

#[test]
fn option_d_documentation_quotes_are_rendered_verbatim() {
    let schema = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    assert!(markdown.contains("Verbatim quote"));
    assert!(markdown.contains("Valid Range: Minimum value of 1. Maximum value of 60."));
    assert!(markdown.contains("2026-11-07"));
    assert!(markdown.contains("HISTORICAL \\| FORECAST"));
    assert!(markdown.contains("` Valid Values: HISTORICAL \\| FORECAST `"));
    assert!(markdown.contains("` Pattern: ^(?!"));
    assert!(markdown.contains("<script>"));
    assert!(!markdown.contains("Not recorded"));
    assert!(!markdown.contains("No source recorded"));
}

#[test]
fn option_d_markdown_keeps_global_enum_evidence() {
    for resource in [
        "AWS::EC2::EIP",
        "AWS::EC2::IPAMPool",
        "AWS::EC2::NatGateway",
        "AWS::EC2::Subnet",
        "AWS::EC2::TransitGateway",
        "AWS::EC2::TransitGatewayAttachment",
    ] {
        let schema = cfn_schema_for_codegen_tests(resource);
        generate_schema_code(&schema, resource).unwrap();
        let mut expected = overrides::global_narrowings().into_keys().collect::<BTreeSet<_>>();
        expected.extend(resource_type_overrides().for_resource(resource).filter_map(|(path, entry)| {
            entry.narrowing.as_ref().map(|_| path.source_pointer())
        }));
        let markdown = generate_test_markdown(&schema, resource).unwrap();
        let actual = markdown.lines().filter_map(|line| {
            line.strip_prefix("| `")?.split_once("` | Subset:").map(|(path, _)| path.to_string())
        }).collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "{resource}");
    }
}

#[test]
fn option_d_documented_restorations_and_drops_preserve_cfn() {
    let identity = cfn_schema_for_codegen_tests("AWS::SSO::Instance");
    let rust = generate_schema_code(&identity, &identity.type_name).unwrap();
    assert!(rust.contains("d-[0-9a-f]{10}"), "{rust}");
    let route53 = cfn_schema_for_codegen_tests("AWS::Route53::HostedZone");
    let rust = generate_schema_code(&route53, &route53.type_name).unwrap();
    assert!(rust.contains("Some((None, Some(32)))"), "{rust}");
    let sg = cfn_schema_for_codegen_tests("AWS::EC2::SecurityGroupIngress");
    let rust = generate_schema_code(&sg, &sg.type_name).unwrap();
    let protocol = rust.split("fn build_ip_protocol_attribute()").nth(1).unwrap().split("schema = schema.attribute").next().unwrap();
    assert!(protocol.contains("AttributeType::enum_("), "{protocol}");
    let pool = cfn_schema_for_codegen_tests("AWS::EC2::IPAMPool");
    let rust = generate_schema_code(&pool, &pool.type_name).unwrap();
    assert!(rust.contains("\"ipv4\", \"ipv6\""), "{rust}");
    assert!(!rust.contains("\"IPv4\", \"IPv6\""), "{rust}");
}

#[test]
fn option_d_policy_grammar_keeps_documented_structure_and_existing_spelling() {
    let schema = cfn_schema_for_codegen_tests("AWS::IAM::Role");
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    assert!(rust.contains("carina_aws_types::iam_policy_document()"), "{rust}");
    assert!(rust.contains(".with_documented_iam_grammar()"), "{rust}");
    assert!(!rust.contains("__CfnJsonValue"), "{rust}");
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    assert!(markdown.contains("PolicyDocument"), "{markdown}");
}

#[test]
fn option_d_identity_keeps_each_fields_cfn_constraints() {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(
        ("AWS::Test::Emission", "Probe"),
        TypeOverride::identity(Some(StringIdentity::Arn)),
    );
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission",
            "properties":{"Probe":{"type":"string","pattern":"[a-z]+","minLength":2,"maxLength":9}}
        }))
        .unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("AttributeType::string_member("), "{rust}");
        assert!(rust.contains("Some((Some(2), Some(9)))"), "{rust}");
        assert!(rust.contains("[a-z]+"), "{rust}");
    });
}

#[test]
fn option_d_core_builtin_preserves_member_constraints_in_rust_and_docs() {
    let schema = cfn_schema_for_codegen_tests("AWS::IAM::OIDCProvider");
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    assert!(rust.contains("StringIdentity::HttpsUrl.identity_layer(), carina_core::schema::StringMemberConstraints::new(None, Some((Some(1), Some(255))))"), "{rust}");
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    assert!(markdown.contains("HttpsUrl — identity: String; member: String(len: 1..=255)"), "{markdown}");
    assert!(!markdown.contains("not enforced locally"), "{markdown}");
}

fn normalized(pattern: &str) -> StringConstraints {
    string_constraints(
        &Projection {
            types: vec![schema_layer::JsonType::String],
            pattern: Some(pattern.into()),
            ..Projection::default()
        },
        "AWS::Test::Emission",
        CfnPropertyPath::new(None, "Probe"),
        "/properties/Probe",
    )
}

fn accepts(constraints: &StringConstraints, value: &str) -> bool {
    let n = value.chars().count() as u64;
    constraints.min_length.is_none_or(|min| n >= min)
        && constraints.max_length.is_none_or(|max| n <= max)
        && constraints.pattern.as_ref().is_none_or(|p| {
            carina_core::schema::CompiledPattern::try_new(p.as_spec())
                .unwrap()
                .is_match(value)
        })
}

#[test]
fn option_d_duration_lookahead_equivalence() {
    let source = r"^(-?)P(?=\d|T\d)(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)([DW]))?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+(?:\.\d+)?)S)?)?$";
    let original = // ECMAScript \d is ASCII; fancy-regex otherwise uses Unicode digits.
    fancy_regex::Regex::new(&source.replace(r"\d", "[0-9]")).unwrap();
    let rewritten = normalized(source);
    for value in [
        "",
        "P",
        "PT",
        "-P",
        "P0D",
        "P1Y2M3D",
        "P1W",
        "P1WT",
        "PT0S",
        "PT1H",
        "PT12H",
        "PT12H0.1S",
        "-PT1H",
        "PT.5S",
        "PT0.5S",
        "P1DT",
        "P1DT2H3M4.5S",
        "P1D2W",
        "P1.5D",
        "PT1M2H",
        "PT1S\n",
        "PT١H",
    ] {
        assert_eq!(
            accepts(&rewritten, value),
            original.is_match(value).unwrap(),
            "{value:?}"
        );
    }
}

#[test]
fn option_d_forwarded_header_reviewed_original_meaning() {
    // Original CFN intent table: 1..=255 ASCII letters, digits or hyphens.
    // The stacked quantifier is not ECMAScript. fancy-regex instead reads
    // the braces literally, so it cannot be an oracle for the intended rule.
    let source = "^[a-zA-Z0-9-]+{1,255}$";
    let rewritten = normalized(source);
    for (value, expected) in [
        ("".into(), false),
        ("a".into(), true),
        ("X-Forwarded-For".into(), true),
        ("-".repeat(255), true),
        ("a".repeat(256), false),
        ("x_y".into(), false),
        ("x:y".into(), false),
        ("é".into(), false),
        ("x\n".into(), false),
    ] {
        assert_eq!(
            accepts(&rewritten, &value),
            expected,
            "rewritten: {value:?}"
        );
    }
}

#[test]
fn option_d_json_pointer_reviewed_original_meaning() {
    // Original CFN intent table: JSON pointer syntax with 1..=512 characters.
    // As above, the malformed stacked quantifier has no ECMAScript language;
    // fancy-regex interprets its braces as a literal suffix.
    let source = r"^[\/]+([^~]*(~[01])*)*{1,512}$";
    let rewritten = normalized(source);
    for (value, expected) in [
        ("".into(), false),
        ("/".into(), true),
        ("//a/~0/~1".into(), true),
        ("/~01".into(), true),
        ("/~10".into(), true),
        ("/~2".into(), false),
        ("/~".into(), false),
        ("a/b".into(), false),
        (format!("/{}", "a".repeat(511)), true),
        (format!("/{}", "a".repeat(512)), false),
        ("/é\n".into(), true),
    ] {
        assert_eq!(
            accepts(&rewritten, &value),
            expected,
            "rewritten: {value:?}"
        );
    }
}

fn constrained_kms_log_group_schema() -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName": "AWS::Logs::LogGroup",
        "properties": {
            "LogGroupName": {
                "type": "string",
                "pattern": r"^[.\-_/#A-Za-z0-9]{1,512}\Z"
            },
            "KmsKeyId": {
                "type": "string",
                "pattern": r"^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key|alias)/.+\Z",
                "maxLength": 256
            }
        },
        "required": ["LogGroupName"],
        "primaryIdentifier": ["/properties/LogGroupName"]
    }))
    .unwrap()
}

#[test]
fn non_replacing_kms_overrides_preserve_reference_input_rules() {
    for (case, entry) in [
        (
            "identity-only",
            TypeOverride::identity(Some(StringIdentity::Arn)),
        ),
        (
            "string Primitive refinement",
            TypeOverride::identity(Some(StringIdentity::Arn)).with_narrowing(
                NarrowingKind::Primitive {
                    expression: ConstraintSpec::string(
                        None,
                        Some((Some(1), Some(256))),
                        PatternLiteral::Quoted,
                    ),
                    display: OverrideTypeDisplay::new("String"),
                    block_name: None,
                    default: None,
                },
                AwsEvidence::fixture(),
            ),
        ),
    ] {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::Logs::LogGroup", "KmsKeyId"), entry);
        with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
            let schema = constrained_kms_log_group_schema();
            let context = EmissionContext::from_source(&schema).unwrap();
            assert!(
                !context.reference_input_rules().is_empty(),
                "{case} must preserve Reference input rules",
            );
            let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
            let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
            assert!(rust.contains("carina_aws_types::kms_key_reference()"), "{case}: {rust}");
            assert!(rust.contains("schema.input_rule("), "{case}: {rust}");
            assert!(markdown.contains("aws.kms.Key.Reference"), "{case}: {markdown}");
            assert!(
                markdown.contains("## Additional Input Constraints"),
                "{case}: {markdown}",
            );
        });
    }
}

#[test]
fn deprecated_kms_property_keeps_runtime_rules_and_documented_rows() {
    let overrides = Box::leak(Box::new(ResourceTypeOverrides::default()));
    with_resource_type_overrides(overrides, || {
        let mut schema = constrained_kms_log_group_schema();
        schema.edit_json(|source| {
            source["properties"]["KmsKeyId"]["description"] =
                json!("(Deprecated.) The KMS key identifier.");
        });
        assert!(is_deprecated_property(&schema.properties["KmsKeyId"]));
        let context = EmissionContext::from_source(&schema).unwrap();
        let rust = super::super::render_schema_code(&context, &schema.type_name).unwrap();
        let rules = context.reference_input_rules();
        assert!(!rules.is_empty(), "deprecated runtime slots retain their rules");
        assert!(rust.contains("build_kms_key_id_attribute"), "{rust}");
        assert!(rust.contains("carina_aws_types::kms_key_reference()"), "{rust}");
        assert_eq!(rust.matches("schema.input_rule(").count(), rules.len(), "{rust}");

        // Render Markdown with a fresh context, independent of the Rust visit.
        let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
        let mut section = carina_docgen::PageBuilder::<()>::new();
        section.input_rules_section(&rules).unwrap();
        let expected = section.finish().unwrap();
        assert_eq!(markdown.matches(&expected).count(), 1, "{markdown}\nExpected:\n{expected}");
        assert!(!markdown.contains("### `kms_key_id`\n"), "{markdown}");
    });
}

#[test]
fn create_only_kms_property_derives_reference_rules_in_both_outputs() {
    let overrides = Box::leak(Box::new(ResourceTypeOverrides::default()));
    with_resource_type_overrides(overrides, || {
        let mut schema = constrained_kms_log_group_schema();
        schema.edit_json(|source| {
            source["createOnlyProperties"] = json!(["/properties/KmsKeyId"]);
        });
        let context = EmissionContext::from_source(&schema).unwrap();
        let rules = context.reference_input_rules();
        assert!(!rules.is_empty(), "must derive rules for the constrained KMS slot");
        for rule in &rules {
            assert_eq!(
                rule.scope(),
                carina_core::schema::InputRuleScope::CreateOnly,
                "{} must inherit the CFN createOnlyProperties scope",
                rule.code(),
            );
        }
        let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
        assert_eq!(
            rust.matches("InputRuleScope::CreateOnly").count(),
            rules.len(),
            "{rust}",
        );
        let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
        let rows: Vec<_> = markdown
            .lines()
            .filter(|line| line.starts_with("| `value at kms_key_id: "))
            .collect();
        assert_eq!(rows.len(), rules.len(), "{markdown}");
        for row in rows {
            assert!(row.ends_with("(on create)` |"), "{row}");
        }
    });
}

#[test]
fn type_replacing_kms_override_has_no_reference_input_rules() {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(
        ("AWS::Logs::LogGroup", "KmsKeyId"),
        NarrowingKind::NamedType {
            name: "OverriddenKmsKey",
            expression: ConstraintSpec::string(None, None, PatternLiteral::Quoted),
            display: OverrideTypeDisplay::new("OverriddenKmsKey"),
        },
    );
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema = constrained_kms_log_group_schema();
        let context = EmissionContext::from_source(&schema).unwrap();
        assert!(
            context.reference_input_rules().is_empty(),
            "a NamedType override must not declare Reference input rules",
        );
        let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
        let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
        assert!(
            rust.contains("AttributeType::ref_(\"OverriddenKmsKey\".to_string())"),
            "{rust}",
        );
        assert!(
            markdown.contains("### `kms_key_id`\n\n- **Type:** `OverriddenKmsKey`\n"),
            "{markdown}",
        );
        assert!(!rust.contains("kms_key_reference()"), "{rust}");
        assert!(!rust.contains("schema.input_rule("), "{rust}");
        assert!(!markdown.contains("aws.kms.Key.Reference"), "{markdown}");
        assert!(
            !markdown.contains("## Additional Input Constraints"),
            "{markdown}",
        );
    });
}

#[test]
fn option_d_identity_and_narrowing_are_independent() {
    let entry = TypeOverride::identity(Some(StringIdentity::Arn)).with_narrowing(
        NarrowingKind::Primitive {
            expression: ConstraintSpec::string(
                None,
                Some((Some(2), Some(5))),
                PatternLiteral::Quoted,
            ),
            display: OverrideTypeDisplay::new("String"),
            block_name: None,
            default: None,
        },
        AwsEvidence::fixture(),
    );
    assert_eq!(entry.identity, Some(StringIdentity::Arn));
    assert!(entry.narrowing.is_some());
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission", "Probe"), entry);
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema =
            ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
                "Probe":{"type":"string","pattern":"^[a-z]+$","minLength":3,"maxLength":9}
            }}))
            .unwrap();
        let output = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            output.contains("AttributeType::string_member("),
            "{output}"
        );
        assert!(output.contains("Some((Some(3), Some(5)))"), "{output}");
        assert!(output.contains("[a-z]+"), "{output}");
    });
}

#[test]
fn option_d_narrowing_rejects_a_different_kind() {
    let entry = TypeOverride::identity(None).with_narrowing(
        NarrowingKind::Primitive {
            expression: ConstraintSpec::int_range(Some(1), Some(5)),
            display: OverrideTypeDisplay::new("Int"),
            block_name: None,
            default: None,
        },
        AwsEvidence::fixture(),
    );
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission", "Probe"), entry);
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema =
            ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
                "Probe":{"type":"string"}
            }}))
            .unwrap();
        assert!(generate_synthetic_schema_code(&schema).is_err());
        assert!(generate_test_markdown(&schema, &schema.type_name).is_err());
    });
}

#[test]
fn option_d_pattern_narrowing_cannot_discard_the_cfn_pattern() {
    let entry = TypeOverride::identity(None).with_narrowing(
        NarrowingKind::Primitive {
            expression: ConstraintSpec::string(
                Some(ApprovedPattern::whole("^[a-z]+$")),
                None,
                PatternLiteral::Quoted,
            ),
            display: OverrideTypeDisplay::new("String"),
            block_name: None,
            default: None,
        },
        AwsEvidence::fixture(),
    );
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission", "Probe"), entry);
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema =
            ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
                "Probe":{"type":"string","pattern":"^a+$"}
            }}))
            .unwrap();
        let output = generate_synthetic_schema_code(&schema).unwrap();
        assert!(output.contains("^a+$"), "{output}");
        assert!(!output.contains("^[a-z]+$"), "{output}");
    });
}

#[test]
fn option_d_waf_description_intersection_preserves_both_patterns() {
    use override_type_expression::{
        WAF_DESCRIPTION_API, WAF_DESCRIPTION_CFN, WAF_DESCRIPTION_INTERSECTION,
    };
    let compile = |pattern| {
        carina_core::schema::CompiledPattern::try_new(
            &carina_core::schema::PatternSpec::new(pattern, PatternMatch::Full),
        ).unwrap()
    };
    let cfn = compile(WAF_DESCRIPTION_CFN);
    let api = compile(WAF_DESCRIPTION_API);
    let actual = compile(WAF_DESCRIPTION_INTERSECTION);
    for value in [
        "",
        "a",
        "ab",
        "abc",
        "a b",
        "a\tb",
        "a\nb",
        "_ab",
        "a_b",
        "ab_",
        "a+b",
        "+ab",
        "ab+",
        "a-b",
        "ab-",
        "a\u{0085}b",
        "a b\n",
    ] {
        assert_eq!(
            actual.is_match(value),
            cfn.is_match(value) && api.is_match(value),
            "{value:?}"
        );
    }
}

type RecordedSites =
    BTreeMap<(String, String, String), (Option<StringIdentity>, StringConstraints)>;
thread_local! {
    static SITES: RefCell<RecordedSites> = const { RefCell::new(BTreeMap::new()) };
}

pub(crate) fn record_site(
    resource: &str,
    path: CfnPropertyPath<'_>,
    source: &str,
    identity: Option<StringIdentity>,
    constraints: &StringConstraints,
) {
    SITES.with(|sites| {
        sites.borrow_mut().insert(
            (resource.into(), path.to_string(), source.into()),
            (identity, constraints.clone()),
        );
    });
}

#[test]
fn option_d_capture_actual_sites_and_evidence() {
    SITES.with(|sites| sites.borrow_mut().clear());
    let mut evidence = Vec::new();
    for file in cached_schema_paths() {
        let schema = ParsedCfnSchema::from_json(
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap(),
        )
        .unwrap();
        if !schema.is_cloud_control_provisionable() {
            continue;
        }
        generate_schema_code(&schema, &schema.type_name)
            .unwrap_or_else(|error| panic!("{}: {error:#}", schema.type_name));
        for (pointer, provenance) in overrides::global_narrowings() {
            evidence.push(json!({"resource":schema.type_name,"field":pointer,"evidence":provenance}));
        }
        for (path, entry) in resource_type_overrides().for_resource(&schema.type_name) {
            if let Some(narrowing) = &entry.narrowing {
                evidence.push(json!({"resource":schema.type_name,"field":path.source_pointer(),"kind":format!("{:?}",narrowing.kind),"evidence":narrowing.evidence}));
            }
        }
    }
    let sites = SITES.with(|sites| sites.borrow().iter().map(|((resource,field,source),(identity,c))| {
        let legacy = identity.and_then(StringIdentity::callback_declaration_metadata);
        json!({"resource":resource,"field":field,"source":source,"identity":identity.map(|id|id.identity().to_string()),
            "pattern":c.pattern.as_ref().map(ApprovedPattern::as_str),"minLength":c.min_length,"maxLength":c.max_length,
            "legacy":legacy,"has_callback":identity.is_some_and(|id|id.check().is_some())})
    }).collect::<Vec<_>>());
    evidence.sort_by_key(|entry| {
        (
            entry["resource"].as_str().unwrap().to_string(),
            entry["field"].as_str().unwrap().to_string(),
        )
    });
    let builtin_member_sites = SITES.with(|sites| {
        sites
            .borrow()
            .iter()
            .filter(|(_, (identity, c))| {
                identity.is_some_and(|id| id.is_core_builtin()) && c.has_local_constraints()
            })
            .map(|((resource, field, _), _)| (resource.clone(), field.clone()))
            .collect::<Vec<_>>()
    });
    assert_eq!(
        builtin_member_sites,
        vec![("AWS::IAM::OIDCProvider".into(), "Url".into())]
    );
    verify_loosening_examples(&sites);
    let Some(directory) = std::env::var_os("OPTION_D_REPORT_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    write_loosening_report(&sites, &directory);
    std::fs::write(
        directory.join("602-D-sites.json"),
        serde_json::to_string_pretty(&sites).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("602-D-evidence.json"),
        serde_json::to_string_pretty(&evidence).unwrap(),
    )
    .unwrap();
}

#[test]
fn option_d_callbacks_are_shared_per_identity() {
    for identity in StringIdentity::ALL {
        if let (Some(first), Some(second)) = (identity.check(), identity.check()) {
            assert!(std::sync::Arc::ptr_eq(&first, &second), "{identity:?}");
        }
    }
}

#[test]
fn option_d_referenced_enum_does_not_borrow_another_fields_domain() {
    let schema = ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission",
        "properties":{"Scope":{"type":"string","enum":["ROOT"]},"Settings":{"$ref":"#/definitions/Settings"}},
        "definitions":{"NestedScope":{"type":"string","enum":["LABEL","NAMESPACE"]},
            "Settings":{"type":"object","properties":{"Scope":{"$ref":"#/definitions/NestedScope"}}}}
    })).unwrap();
    let output = generate_synthetic_schema_code(&schema).unwrap();
    assert!(output.contains("LABEL"));
    assert!(output.contains("NAMESPACE"));
    assert!(output.contains("ROOT"));
    generate_test_markdown(&schema, &schema.type_name).unwrap();
}

fn legacy_constraints(metadata: &serde_json::Value) -> StringConstraints {
    let length = metadata[1].as_array();
    StringConstraints {
        pattern: metadata[0].as_str().map(ApprovedPattern::whole),
        min_length: length.and_then(|n| n[0].as_u64()),
        max_length: length.and_then(|n| n[1].as_u64()),
    }
}

fn site_string_identity(site: &serde_json::Value) -> Option<StringIdentity> {
    site["identity"].as_str().map(|name| {
        let context = format!(
            "{}.{} must have a canonical declaration for site identity {name}",
            site["resource"].as_str().unwrap(),
            site["field"].as_str().unwrap(),
        );
        StringIdentity::ALL
            .iter()
            .copied()
            .find(|id| id.identity().to_string() == name)
            .expect(&context)
    })
}

fn identity_rejects(
    identity: Option<StringIdentity>,
    value: &carina_core::resource::Value,
) -> bool {
    use carina_core::schema::{AttributeType, StringMemberConstraints, ValidatedConfigSchema};

    identity.is_some_and(|identity| {
        let ty = AttributeType::string_member(
            identity.identity_layer(),
            StringMemberConstraints::new(None, None),
        );
        let schema =
            ValidatedConfigSchema::validate(BTreeMap::from([("value".into(), ty)])).unwrap();
        schema.rootless_type("value").unwrap().validate(value).is_err()
    })
}

fn write_loosening_report(sites: &[serde_json::Value], directory: &std::path::Path) {
    let fallback: serde_json::Value =
        serde_json::from_str(include_str!("option_d_legacy_metadata.json")).unwrap();
    let legacy_sites: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("option_d_legacy_sites.json")).unwrap();
    let mut cases = Vec::new();
    for site in sites {
        // All named AWS strings plus the historical anonymous gateway union.
        // CIDR unions had no old structural pattern/length, only core checks.
        let metadata = if let Some(identity_name) = site["identity"].as_str() {
            if !identity_name.starts_with("aws.") {
                continue;
            }
            let metadata = if site["legacy"].is_null() {
                &fallback[identity_name]
            } else {
                &site["legacy"]
            };
            assert!(
                !metadata.is_null(),
                "missing historical metadata for {identity_name}"
            );
            metadata.clone()
        } else if site["resource"] == "AWS::EC2::Route" && site["field"] == "GatewayId" {
            let patterns = [
                StringIdentity::Ec2InternetGatewayId,
                StringIdentity::Ec2VpnGatewayId,
            ]
            .map(|id| id.callback_declaration_metadata().unwrap().0.unwrap());
            json!([format!(r"^(?:{}|{})\z", patterns[0], patterns[1]), null])
        } else {
            continue;
        };
        let old = legacy_constraints(&metadata);
        let new = StringConstraints {
            pattern: site["pattern"].as_str().map(ApprovedPattern::whole),
            min_length: site["minLength"].as_u64(),
            max_length: site["maxLength"].as_u64(),
        };
        let mut candidates = vec![
            String::new(),
            "x".into(),
            "x".repeat(new.min_length.unwrap_or(1) as usize),
        ];
        for prior in &legacy_sites {
            if prior["resource"] == site["resource"] && prior["pointer"] == site["source"] {
                for key in ["negative", "positive"] {
                    if let Some(value) = prior[key].as_str() {
                        candidates.push(value.to_string());
                    }
                }
            }
        }
        let example = candidates
            .into_iter()
            .find(|value| accepts(&new, value) && !accepts(&old, value))
            .or_else(|| {
                override_type_expression::loosening_example(&new, &old).unwrap_or_else(|error| {
                    panic!("{} {}: {error:#}", site["resource"], site["field"])
                })
            });
        let Some(example) = example else {
            continue;
        };
        assert!(accepts(&new, &example) && !accepts(&old, &example));
        let identity = site_string_identity(site);
        let value = carina_core::resource::Value::Concrete(
            carina_core::resource::ConcreteValue::String(example.clone().into()),
        );
        let rejected = identity
            .and_then(|id| id.check())
            .is_some_and(|check| check(&value).is_err());
        let mut case = site.clone();
        case["oldPattern"] = metadata[0].clone();
        case["oldLength"] = metadata[1].clone();
        case["example"] = json!(example);
        case["callbackRejected"] = json!(rejected);
        case["identityRejected"] = json!(identity_rejects(identity, &value));
        cases.push(case);
    }
    let show = |pattern: &serde_json::Value, min: Option<u64>, max: Option<u64>| {
        format!(
            "pattern {}; length {}..{}",
            pattern
                .as_str()
                .map_or_else(|| "none".into(), |p| format!("`{}`", p.replace('|', "\\|"))),
            min.map_or_else(|| "0".into(), |n| n.to_string()),
            max.map_or_else(|| "unbounded".into(), |n| n.to_string())
        )
    };
    let mut markdown = String::from(
        "# Option D: structurally loosened fields\n\nEach row has a concrete value accepted by the emitted CFN projection and rejected by the historical known type's pattern or length. Shared callbacks are checked independently. The scan covers all emitted named string sites and historical anonymous string unions; core built-ins retain their historical declaration. API narrowings remain intersected with CFN.\n\n| Resource | Field | Old constraint | New CFN constraint | Newly accepted structurally | Callback still rejects? |\n| --- | --- | --- | --- | --- | --- |\n",
    );
    for case in &cases {
        let length = case["oldLength"].as_array();
        markdown.push_str(&format!(
            "| {} | `{}` | {} | {} | `{}` | {} |\n",
            case["resource"].as_str().unwrap(),
            case["field"].as_str().unwrap(),
            show(
                &case["oldPattern"],
                length.and_then(|n| n[0].as_u64()),
                length.and_then(|n| n[1].as_u64())
            ),
            show(
                &case["pattern"],
                case["minLength"].as_u64(),
                case["maxLength"].as_u64()
            ),
            serde_json::to_string(&case["example"])
                .unwrap()
                .replace('|', "\\|"),
            if case["callbackRejected"] == true {
                "Yes"
            } else if case["has_callback"] == true {
                "No"
            } else {
                "No callback"
            }
        ));
    }
    std::fs::write(directory.join("602-D-loosened.md"), markdown).unwrap();
    std::fs::write(
        directory.join("602-D-loosened-cases.json"),
        serde_json::to_string_pretty(&cases).unwrap(),
    )
    .unwrap();
}

#[test]
fn option_d_named_enum_preserves_identity_callback_and_cfn_domain() {
    let a = "arn:aws:example:us-east-1:123456789012:resource/a";
    let b = "arn:aws:example:us-east-1:123456789012:resource/b";
    let c = "arn:aws:example:us-east-1:123456789012:resource/c";
    let entry = TypeOverride::identity(Some(StringIdentity::Arn)).with_narrowing(
        NarrowingKind::Enum(vec![a, b]),
        AwsEvidence::fixture(),
    );
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission", "Probe"), entry);
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema = ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission","properties":{
            "Probe":{"type":"string","pattern":"^arn:aws:example:us-east-1:[0-9]{12}:resource/[abc]$","minLength":20,"maxLength":128,"enum":[a,b,c]}
        }})).unwrap();
        let output = generate_synthetic_schema_code(&schema).unwrap();
        assert!(output.contains("StringIdentity::Arn.identity_layer()"));
        assert!(output.contains("StringMemberConstraints::new("));
        let property = &schema.properties["Probe"];
        let constraints =
            resource_type_override(&schema.type_name, CfnPropertyPath::new(None, "Probe"))
                .unwrap()
                .named_literal_constraints(property, &schema, CfnPropertyPath::new(None, "Probe"))
                .unwrap()
                .unwrap();
        for (value, expected) in [(a, true), (b, true), (c, false)] {
            assert_eq!(accepts(&constraints, value), expected);
            let value = carina_core::resource::Value::Concrete(
                carina_core::resource::ConcreteValue::String(value.into()),
            );
            assert!(StringIdentity::Arn.check().unwrap()(&value).is_ok());
        }
        generate_test_markdown(&schema, &schema.type_name).unwrap();
    });
}

#[test]
fn option_d_failed_containment_is_a_schema_error() {
    for (source, kind, identity) in [
        (
            json!({"type":"string","minLength":3}),
            NarrowingKind::Enum(vec!["a"]),
            None,
        ),
        (
            json!({"type":"string","pattern":"^a+$"}),
            NarrowingKind::Enum(vec!["bbb"]),
            None,
        ),
        (
            json!({"type":"string"}),
            NarrowingKind::Primitive {
                expression: ConstraintSpec::document(DocumentKind::SnsSubscriptionReplayPolicy),
                display: OverrideTypeDisplay::new("PolicyDocument"),
                block_name: None,
                default: None,
            },
            None,
        ),
        (
            json!({"type":"array","items":{"type":"integer"}}),
            NarrowingKind::ArrayItemType {
                expression: ConstraintSpec::int_range(Some(1), Some(3)),
                display: OverrideTypeDisplay::new("List<Int>"),
                block_name: None,
                default: None,
                min_items: None,
                max_items: None,
            },
            Some(StringIdentity::Arn),
        ),
    ] {
        let entry =
            TypeOverride::identity(identity).with_narrowing(kind, AwsEvidence::fixture());
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::Test::Emission", "Probe"), entry);
        with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
            let schema = ParsedCfnSchema::from_json(
                json!({"typeName":"AWS::Test::Emission","properties":{"Probe":source}}),
            )
            .unwrap();
            for error in [
                generate_synthetic_schema_code(&schema).unwrap_err(),
                generate_test_markdown(&schema, &schema.type_name).unwrap_err(),
            ] {
                assert!(
                    error.downcast_ref::<schema_layer::SchemaError>().is_some(),
                    "{error:#}"
                );
            }
        });
    }
}

/// The fixture records historical constraints, a newly admitted value and the
/// current callback/identity rejection facts. Match each case to today's
/// projected site instead of trusting cached new constraints in the fixture.
fn verify_loosening_examples(sites: &[serde_json::Value]) {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("option_d_loosened_cases.json")).unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let site = sites
            .iter()
            .find(|site| {
                site["resource"] == case["resource"]
                    && site["source"] == case["source"]
                    && site["field"] == case["field"]
            })
            .unwrap_or_else(|| panic!("loosened fixture no longer emitted: {case}"));
        let current = StringConstraints {
            pattern: site["pattern"].as_str().map(ApprovedPattern::whole),
            min_length: site["minLength"].as_u64(),
            max_length: site["maxLength"].as_u64(),
        };
        let old = legacy_constraints(&json!([case["oldPattern"], case["oldLength"]]));
        let example = case["example"].as_str().unwrap();
        assert!(accepts(&current, example), "new constraints reject {case}");
        assert!(
            !accepts(&old, example),
            "historical constraints accept {case}"
        );
        let identity = site_string_identity(site);
        let check = identity.and_then(|id| id.check());
        assert_eq!(
            check.is_some(),
            case["has_callback"].as_bool().unwrap(),
            "{case}"
        );
        let value = carina_core::resource::Value::Concrete(
            carina_core::resource::ConcreteValue::String(example.to_string().into()),
        );
        assert_eq!(
            check.is_some_and(|check| check(&value).is_err()),
            case["callbackRejected"].as_bool().unwrap(),
            "{case}"
        );
        assert_eq!(
            Some(identity_rejects(identity, &value)),
            case["identityRejected"].as_bool(),
            "identity rejection must match the recorded fact for {case}",
        );
    }
}

#[test]
fn option_d_rejection_baseline_is_preserved() {
    // Callback rejections recorded at 317ef6b5ab91eef16f81d718cd2f4fa07d7adf48.
    // Keep this baseline independent of the regenerable loosening report.
    let baseline: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("option_d_rejection_baseline.json")).unwrap();
    assert!(!baseline.is_empty());
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("option_d_loosened_cases.json")).unwrap();
    for baseline_case in baseline {
        let resource = baseline_case["resource"].as_str().unwrap();
        let field = baseline_case["field"].as_str().unwrap();
        let example = baseline_case["example"].as_str().unwrap();
        let context = format!(
            "rejection baseline case {resource}.{field} example {example:?} must remain in the loosened-cases fixture"
        );
        let case = cases
            .iter()
            .find(|case| {
                case["resource"] == resource
                    && case["field"] == field
                    && case["example"] == example
            })
            .expect(&context);
        assert!(
            case["callbackRejected"] == true || case["identityRejected"] == true,
            "rejection baseline case {resource}.{field} example {example:?} no longer rejects: {case}",
        );
    }
}

#[test]
fn option_d_rewrite_table_covers_exactly_four_cached_sites() {
    fn visit(value: &serde_json::Value, found: &mut BTreeMap<String, usize>) {
        match value {
            serde_json::Value::Object(object) => {
                if let Some(pattern) = object.get("pattern").and_then(serde_json::Value::as_str)
                    && CFN_PATTERN_NORMALIZATIONS
                        .iter()
                        .any(|rule| rule.source == pattern)
                {
                    *found.entry(pattern.to_string()).or_default() += 1;
                }
                for child in object.values() {
                    visit(child, found);
                }
            }
            serde_json::Value::Array(array) => {
                for child in array {
                    visit(child, found);
                }
            }
            _ => {}
        }
    }
    let mut found = BTreeMap::new();
    for file in cached_schema_paths() {
        visit(
            &serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap(),
            &mut found,
        );
    }
    assert_eq!(
        CFN_PATTERN_NORMALIZATIONS
            .iter()
            .map(|rule| found.get(rule.source).copied().unwrap_or(0))
            .collect::<Vec<_>>(),
        vec![2, 1, 1]
    );
}

#[test]
fn option_d_top_level_identity_does_not_leak_to_a_nested_same_name() {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(
        ("AWS::Test::Emission", "Probe"),
        TypeOverride::identity(Some(StringIdentity::SsoInstanceArn)),
    );
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let schema=ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission","properties":{
            "Probe":{"type":"string","pattern":"^top$"},
            "Config":{"type":"object","properties":{"Probe":{"type":"string","pattern":"^[a-z]+$"}}}
        }})).unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert_eq!(
            rust.matches("StringIdentity::SsoInstanceArn.identity_layer()")
                .count(),
            1,
            "{rust}"
        );
        assert!(rust.contains("StructField::optional(\"probe\", AttributeType::refined_string(None, Some(carina_core::schema::PatternSpec::new(\"^[a-z]+$\", carina_core::schema::PatternMatch::Full))"),"{rust}");
        generate_test_markdown(&schema, &schema.type_name).unwrap();
    });
}
