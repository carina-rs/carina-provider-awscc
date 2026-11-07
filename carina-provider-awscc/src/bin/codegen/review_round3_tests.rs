use super::*;
use serde_json::json;

fn projection_errors(schema: &ParsedCfnSchema, pointer: &str, reason: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for (renderer, output) in [
        ("Rust", generate_schema_code(schema, &schema.type_name)),
        (
            "Markdown",
            generate_test_markdown(schema, &schema.type_name),
        ),
    ] {
        let rejected = output.as_ref().is_err_and(|error| {
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer: actual, reason: why })
                    if actual == pointer && why.contains(reason))
        });
        if !rejected {
            failures.push(format!(
                "{renderer} must reject {pointer} ({reason}): {}",
                output.err().map_or_else(
                    || "successful emission".into(),
                    |error| format!("{error:#}")
                )
            ));
        }
    }
    failures
}

#[test]
fn review_r9_mixed_type_one_of_rejects_overlap_before_selecting_an_arm() {
    let mut failures = Vec::new();
    for (selected, bare) in [
        (json!({"type":"string","pattern":"^a$"}), "string"),
        (
            json!({"type":"array","items":{"type":"string"},"minItems":2}),
            "array",
        ),
    ] {
        let arms = [selected, json!({"type":bare}), json!({"type":"object"})];
        for order in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            let mut source = cfn_schema_for_codegen_tests("AWS::EC2::VPC").source;
            source["properties"]["Probe"] = json!({"oneOf":order.map(|index| arms[index].clone())});
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            failures.extend(projection_errors(
                &schema,
                "/properties/Probe",
                "overlapping",
            ));
        }
    }
    // Even discarded arms must remain exclusive with each other.
    let mut source = cfn_schema_for_codegen_tests("AWS::EC2::VPC").source;
    source["properties"]["Probe"] =
        json!({"oneOf":[{"type":"string","pattern":"^a$"},{"type":"object"},{"type":"object"}]});
    let schema = ParsedCfnSchema::from_json(source).unwrap();
    failures.extend(projection_errors(
        &schema,
        "/properties/Probe",
        "overlapping",
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r9_scalar_enum_override_rejects_array_source_in_both_outputs() {
    let mut source = cfn_schema_for_codegen_tests("AWS::S3::Bucket").source;
    source["definitions"]["LambdaConfiguration"]["properties"]["Event"] =
        json!({"type":"array","items":{"type":"string"},"minItems":2});
    let schema = ParsedCfnSchema::from_json(source).unwrap();
    let failures = projection_errors(
        &schema,
        "/definitions/LambdaConfiguration/properties/Event",
        "scalar string",
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r9_document_enum_field_rejects_array_source_in_both_outputs() {
    let mut source = cfn_schema_for_codegen_tests("AWS::S3::Bucket").source;
    let mut destination = source["definitions"]["Destination"].clone();
    let format = destination["properties"]["Format"].clone();
    destination["properties"]["Format"] = json!({"type":"array","items":format,"minItems":2});
    source["definitions"]["DataExport"]["properties"]["Destination"] = destination;
    let schema = ParsedCfnSchema::from_json(source).unwrap();
    let failures = projection_errors(
        &schema,
        "/definitions/DataExport/properties/Destination",
        "scalar string",
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
