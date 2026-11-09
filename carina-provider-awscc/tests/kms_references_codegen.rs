//! Offline checks for fail-closed KMS classification and generated reference leaves.
#![cfg(not(target_arch = "wasm32"))]

use carina_core::schema::Shape;
use serde_json::{Value, json};
use std::process::{Command, Output};

fn generate(schema: &Value, format: &str) -> Output {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), schema.to_string()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_codegen"))
        .args([
            "--type-name",
            schema["typeName"].as_str().unwrap(),
            "--file",
        ])
        .arg(file.path())
        .args(["--format", format, "--examples-dir"])
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
        .output()
        .unwrap()
}

fn vpc_schema() -> Value {
    serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json")).unwrap()
}

#[test]
fn unclassified_kms_strings_fail_rust_and_markdown_generation() {
    for (name, property, path) in [
        ("KmsFoo", json!({"type": "string"}), "kms_foo"),
        ("CmkFoo", json!({"type": "string"}), "cmk_foo"),
        (
            "EncryptionKey",
            json!({"type": "string", "description": "The ARN of an AWS KMS key."}),
            "encryption_key",
        ),
        (
            "EncryptionKey",
            json!({"type": "string", "pattern": "^arn:aws:kms:.*$"}),
            "encryption_key",
        ),
    ] {
        let mut schema = vpc_schema();
        schema["properties"][name] = property;
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(
                !output.status.success(),
                "unclassified {name} passed {format}"
            );
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("unclassified KMS string ec2.Vpc.{path}")),
                "{error}"
            );
        }
    }
}

#[test]
fn classification_follows_shared_definitions_and_array_paths_before_overrides() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__S3__Bucket.json")).unwrap();
    schema["definitions"]["EncryptionConfiguration"]["properties"]["KmsFoo"] =
        json!({"$ref": "#/definitions/SharedKmsString"});
    schema["definitions"]["SharedKmsString"] = json!({"type": "string"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "unclassified nested KmsFoo passed {format}"
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("unclassified KMS string s3.Bucket.replication_configuration.rules[].destination.encryption_configuration.kms_foo"), "{error}");
    }
}

#[test]
fn scheduler_generates_cfn_patterned_reference() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Scheduler__Schedule.json"
    ))
    .unwrap();
    let output = generate(&schema, "rust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pattern = serde_json::to_string(&schema["properties"]["KmsKeyArn"]["pattern"]).unwrap();
    let expected = format!(
        "carina_core::schema::AttributeType::reference(carina_aws_types::AwsType::KmsKeyReference.identity(), Some(carina_core::schema::PatternSpec::new({pattern}, carina_core::schema::PatternMatch::Full)), Some((Some(1), Some(2048))))"
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(&expected)
    );
}

#[test]
fn final_scheduler_leaf_is_a_reference_accepting_alias_arns() {
    let config =
        carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config();
    let schema = config.schema.as_schema();
    let attribute = schema.attributes().get("kms_key_arn").unwrap();
    let Shape::Reference {
        identity,
        pattern: Some(pattern),
        length,
        ..
    } = schema.shape_of(attribute.attr_type())
    else {
        panic!("scheduler kms_key_arn must be a reference leaf");
    };
    assert_eq!(identity.to_string(), "aws.kms.Key.Reference");
    assert_eq!(length, Some((Some(1), Some(2048))));
    let pattern = carina_core::schema::CompiledPattern::try_new(pattern).unwrap();
    assert!(pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/customer"));
    assert!(
        pattern.is_match(
            "arn:aws:kms:us-east-1:123456789012:key/11111111-1111-1111-1111-111111111111"
        )
    );
    assert!(!pattern.is_match("alias/customer"));
}

#[test]
fn scheduler_reference_rejects_alias_names_disallowed_by_cfn() {
    let config =
        carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config();
    let schema = config.schema.as_schema();
    let ty = schema.attributes()["kms_key_arn"].attr_type();
    let Shape::Reference {
        pattern: Some(pattern),
        ..
    } = schema.shape_of(ty)
    else {
        panic!("scheduler kms_key_arn must be a patterned reference");
    };
    let pattern = carina_core::schema::CompiledPattern::try_new(pattern).unwrap();
    assert!(pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/scheduler"));
    assert!(pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/"));
    assert!(!pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/team/scheduler"));
    assert!(!pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/team.scheduler"));
    assert!(!pattern.is_match("arn:aws:kms:us-east-1:١٢٣٤٥٦٧٨٩٠١٢:alias/scheduler"));
}

#[test]
fn changed_kms_pattern_requires_reclassification() {
    let mut schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Scheduler__Schedule.json"
    ))
    .unwrap();
    schema["properties"]["KmsKeyArn"]["pattern"] = json!("^arbitrary-new-constraint$");
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("KMS pattern changed"));
    }
}

#[test]
fn reference_inside_list_struct_preserves_the_array_constraints() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__S3__Bucket.json")).unwrap();
    let array = &mut schema["definitions"]["BucketEncryption"]["properties"]["ServerSideEncryptionConfiguration"];
    assert_eq!(array["type"], "array");
    array["minItems"] = json!(1);
    array["maxItems"] = json!(3);
    array["uniqueItems"] = json!(true);
    array["insertionOrder"] = json!(false);
    let output = generate(&schema, "rust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let code = String::from_utf8(output.stdout).unwrap();
    assert!(code.contains("carina_aws_types::kms_key_reference()"));
    assert!(
        code.contains(
            "false, Some((Some(1), Some(3))), carina_core::schema::ListElements::Unique)"
        )
    );
}

#[test]
fn shared_scalar_description_is_checked_at_the_input_path() {
    let mut schema = vpc_schema();
    schema["properties"]["EncryptionKey"] = json!({"$ref": "#/definitions/SharedString"});
    schema["definitions"]["SharedString"] =
        json!({"type": "string", "description": "The ARN of an AWS KMS key."});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unclassified KMS string ec2.Vpc.encryption_key")
        );
    }
}

#[test]
fn service_name_evidence_in_shared_strings_requires_classification() {
    for evidence in [
        json!({"description": "The AWS Key Management Service key identifier"}),
        json!({"description": "The AWS kEy MaNaGeMeNt SeRvIcE key identifier"}),
        json!({"description": "The AWS kEy\t MaNaGeMeNt\nSeRvIcE key identifier"}),
        json!({"description": "The Customer\r\nMaster\tKey identifier"}),
        json!({"pattern": "Key Management Service"}),
        json!({"pattern": "Key\u{a0}Management\u{2003}Service"}),
    ] {
        let mut schema: Value = serde_json::from_str(include_str!(
            "../../cfn-schema-cache/AWS__ECR__Repository.json"
        ))
        .unwrap();
        schema["definitions"]["EncryptionConfiguration"]["properties"]["EncryptionKey"] =
            json!({"$ref": "#/definitions/SharedString"});
        schema["definitions"]["SharedString"] = json!({"type": "string"});
        schema["definitions"]["SharedString"]
            .as_object_mut()
            .unwrap()
            .extend(evidence.as_object().unwrap().clone());
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "unclassified {evidence} passed {format}"
            );
            assert!(
                error.contains(
                    "unclassified KMS string ecr.Repository.encryption_configuration.encryption_key"
                ),
                "{format}: {error}"
            );
        }
    }
}

#[test]
fn literal_evidence_in_shared_strings_requires_classification() {
    for constraint in [
        json!({"enum": ["AES256", "CMK"]}),
        json!({"const": "cMk"}),
        json!({"enum": ["AES256", "Customer\r\nMaster\tKey"]}),
        json!({"const": "CuStOmEr Master Key"}),
        json!({"enum": ["AES256", "Key\u{a0}Management\u{2003}Service"]}),
        json!({"const": "kEy\t MaNaGeMeNt\nSeRvIcE"}),
        json!({"enum": ["AES256", "KMS"]}),
        json!({"enum": ["AES256", "aws:kms"]}),
        json!({"enum": ["AES256", "aws:kms:dsse"]}),
        json!({"const": "aWs:kMs"}),
    ] {
        let mut schema: Value = serde_json::from_str(include_str!(
            "../../cfn-schema-cache/AWS__ECR__Repository.json"
        ))
        .unwrap();
        schema["definitions"]["EncryptionConfiguration"]["properties"]["EncryptionMode"] =
            json!({"$ref": "#/definitions/EncryptionMode"});
        schema["definitions"]["EncryptionMode"] = json!({"type": "string"});
        schema["definitions"]["EncryptionMode"]
            .as_object_mut()
            .unwrap()
            .extend(constraint.as_object().unwrap().clone());
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "unclassified {constraint} passed {format}"
            );
            assert!(
                error.contains(
                    "unclassified KMS string ecr.Repository.encryption_configuration.encryption_mode"
                ),
                "{format}: {error}"
            );
        }
    }
}

#[test]
fn classification_checks_constraints_on_ref_uses() {
    let mut schema = vpc_schema();
    schema["properties"]["EncryptionKey"] = json!({
        "$ref": "#/definitions/SharedString", "pattern": "^arn:aws:kms:.*$",
    });
    schema["definitions"]["SharedString"] = json!({"type": "string"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "a KMS pattern on a $ref use must be classified"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unclassified KMS string ec2.Vpc.encryption_key")
        );
    }
}

#[test]
fn explicitly_classified_non_reference_scalars_and_key_outputs_keep_their_types() {
    let schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__KMS__Key.json")).unwrap();
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = String::from_utf8(output.stdout).unwrap();
        if format == "rust" {
            assert!(
                output.contains("AttributeSchema::optional(\"enabled\", AttributeType::bool())")
            );
            assert!(output.contains("AttributeSchema::provider_populated(\"arn\", self::arn())"));
            assert!(output.contains(
                "AttributeSchema::provider_populated(\"key_id\", AttributeType::string())"
            ));
        } else {
            assert!(output.contains("### `enabled`\n\n- **Type:** Bool"));
            assert!(output.contains("### `arn`\n\n- **Type:** Arn"));
        }
    }
}

#[test]
fn read_only_parent_does_not_hide_emitted_child_kms_input() {
    let mut schema = vpc_schema();
    schema["properties"]["Settings"] = json!({"type": "object", "properties": {
        "KmsFoo": {"type": "string"}
    }});
    schema["readOnlyProperties"]
        .as_array_mut()
        .unwrap()
        .push(json!("/properties/Settings"));
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "writable child escaped classification in {format}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unclassified KMS string ec2.Vpc.settings.kms_foo")
        );
    }
}

#[test]
fn inferred_string_leaves_cannot_skip_classification() {
    for property in [
        json!({"enum": ["value"]}),
        json!({"enum": [42]}),
        json!({"const": "value"}),
        json!({"const": 42}),
        json!({}),
        json!({"description": "A key identifier"}),
        json!({"anyOf": [{"enum": ["value"]}]}),
        json!({"oneOf": [{"const": "value"}]}),
    ] {
        let mut schema = vpc_schema();
        schema["properties"]["KmsFoo"] = property.clone();
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "{property} passed {format}");
            assert!(
                error.contains("unclassified KMS string ec2.Vpc.kms_foo"),
                "{property}: {error}"
            );
        }
    }
}

#[test]
fn kms_evidence_does_not_require_an_arn_mention() {
    for description in [
        "The ID of the AWS KMS key",
        "The customer master key identifier",
        "The cMk identifier",
    ] {
        let mut schema = vpc_schema();
        schema["properties"]["EncryptionKey"] =
            json!({"type": "string", "description": description});
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "{description} passed {format}");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("unclassified KMS string ec2.Vpc.encryption_key")
            );
        }
    }
}

#[test]
fn constraint_branches_and_ref_siblings_merge_into_the_leaf_evidence() {
    for keyword in ["oneOf", "anyOf", "allOf"] {
        for at_use_site in [false, true] {
            let mut schema = vpc_schema();
            schema["definitions"]["SharedString"] = json!({"type": "string"});
            schema["properties"]["EncryptionKey"] = json!({"$ref": "#/definitions/SharedString"});
            let evidence_site = if at_use_site {
                &mut schema["properties"]["EncryptionKey"]
            } else {
                &mut schema["definitions"]["SharedString"]
            };
            evidence_site[keyword] = json!([{"description": "The ARN of the AWS KMS key"}]);
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                assert!(
                    !output.status.success(),
                    "{keyword}, use={at_use_site} passed {format}"
                );
                assert!(
                    String::from_utf8_lossy(&output.stderr)
                        .contains("unclassified KMS string ec2.Vpc.encryption_key")
                );
            }
        }
    }
}

#[test]
fn key_only_constraint_in_any_schema_branch_requires_reclassification() {
    for keyword in ["oneOf", "anyOf", "allOf"] {
        for typed in [false, true] {
            let mut schema: Value = serde_json::from_str(include_str!(
                "../../cfn-schema-cache/AWS__Scheduler__Schedule.json"
            ))
            .unwrap();
            let mut branch = json!({"pattern": "^arn:aws:kms:.*:key/.*$"});
            if typed {
                branch["type"] = json!("string");
            }
            schema["properties"]["KmsKeyArn"][keyword] = json!([branch]);
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                assert!(
                    !output.status.success(),
                    "{keyword}, typed={typed} passed {format}"
                );
                assert!(String::from_utf8_lossy(&output.stderr).contains("KMS pattern changed"));
            }
        }
    }
}

#[test]
fn read_only_scalar_collections_keep_their_output_types() {
    for property in [
        json!({"type": "array", "items": {"type": "string"}}),
        json!({"type": "object", "patternProperties": {".*": {"type": "string"}}, "additionalProperties": false}),
    ] {
        let mut schema = vpc_schema();
        schema["properties"]["Output"] = property;
        schema["readOnlyProperties"]
            .as_array_mut()
            .unwrap()
            .push(json!("/properties/Output"));
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(
                output.status.success(),
                "{format}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if format == "rust" {
                let code = String::from_utf8(output.stdout).unwrap();
                assert!(code.contains("AttributeSchema::provider_populated"));
                assert!(!code.contains("carina_aws_types::kms_key_reference()"));
            }
        }
    }
}

#[test]
fn projected_tags_walk_the_original_cfn_value_subtree() {
    for tagged in [false, true] {
        let mut schema = vpc_schema();
        schema["definitions"]["Tag"]["properties"]["Value"]["description"] =
            json!("The AWS KMS key");
        if !tagged {
            // Still projected by the Tag $ref, without resource tagging metadata.
            schema.as_object_mut().unwrap().remove("tagging");
        }
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "tagged={tagged} passed {format}");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("unaccounted KMS node ec2.Vpc#/definitions/Tag/properties/Value")
            );
        }
    }
}

#[test]
fn overridden_objects_classify_their_own_evidence_and_hidden_children() {
    for source in [
        json!({"type": "object", "description": "The AWS KMS key"}),
        json!({"type": "object", "anyOf": [{"properties": {
            "Value": {"type": "string", "description": "The AWS KMS key"}
        }}]}),
    ] {
        let mut schema: Value = serde_json::from_str(include_str!(
            "../../cfn-schema-cache/AWS__SNS__Subscription.json"
        ))
        .unwrap();
        schema["properties"]["DeliveryPolicy"] = source.clone();
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "{source} passed {format}");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("unaccounted KMS node sns.Subscription#/properties/DeliveryPolicy")
            );
        }
    }
}

#[test]
fn resource_combinators_contribute_property_constraints_and_candidates() {
    for keyword in ["oneOf", "anyOf", "allOf"] {
        let mut schema: Value =
            serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json"))
                .unwrap();
        schema[keyword] = json!([{"properties": {
            "KmsMasterKeyId": {"pattern": "^arn:aws:kms:.*:key/.*$"}
        }}]);
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "root {keyword} passed {format}");
            assert!(String::from_utf8_lossy(&output.stderr).contains("KMS pattern changed"));
        }

        let mut schema = vpc_schema();
        schema["properties"]["EncryptionKey"] = json!({"type": "string"});
        schema[keyword] = json!([{"allOf": [{"properties": {
            "EncryptionKey": {"description": "The AWS KMS key"}
        }}]}]);
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(
                !output.status.success(),
                "root {keyword} candidate passed {format}"
            );
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("unclassified KMS string ec2.Vpc.encryption_key")
            );
        }
    }
}

#[test]
fn reference_enum_and_const_constraints_require_reclassification() {
    for constraint in [
        json!({"enum": ["alias/only"]}),
        json!({"enum": [42]}),
        json!({"enum": []}),
        json!({"const": "alias/only"}),
        json!({"const": null}),
    ] {
        for location in ["direct", "$ref", "oneOf", "anyOf", "allOf", "root"] {
            let mut schema: Value =
                serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json"))
                    .unwrap();
            match location {
                "direct" => schema["properties"]["KmsMasterKeyId"]
                    .as_object_mut()
                    .unwrap()
                    .extend(constraint.as_object().unwrap().clone()),
                "$ref" => {
                    schema["properties"]["KmsMasterKeyId"] = json!({"$ref": "#/definitions/Key"});
                    schema["definitions"]["Key"] = constraint.clone();
                }
                "root" => schema["allOf"] = json!([{"properties": {"KmsMasterKeyId": constraint}}]),
                keyword => schema["properties"]["KmsMasterKeyId"][keyword] = json!([constraint]),
            }
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                assert!(
                    !output.status.success(),
                    "{location} constraint passed {format}"
                );
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(
                    error.contains("KMS enum/const constraint") && error.contains("revisit"),
                    "{error}"
                );
            }
        }
    }
}
