use super::*;
use serde_json::json;

#[test]
fn review_r10_bounds_only_array_narrowings_preserve_ref_site_constraints() {
    let mut failures = Vec::new();
    for min in [2, 100] {
        for all_of in [false, true] {
            let mut source = cfn_schema_for_codegen_tests("AWS::S3::Bucket").source;
            let refinement = json!({"properties":{"CorsRules":{"minItems":min}}});
            let site = &mut source["properties"]["CorsConfiguration"];
            if all_of {
                site["allOf"] = json!([refinement]);
            } else {
                site["properties"] = refinement["properties"].clone();
            }
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
            let field = rust
                .split_once("StructField::required(\"cors_rules\", ")
                .unwrap()
                .1
                .split_once(".with_provider_name(\"CorsRules\")")
                .unwrap()
                .0;
            let expected = format!(
                "true, Some((Some({min}), Some(100))), carina_core::schema::ListElements::Unique)"
            );
            if !field.contains(&expected) {
                failures.push(format!(
                    "Rust lost CorsRules minItems={min}, allOf={all_of}: expected {expected}"
                ));
            }
            let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
            let row = markdown
                .lines()
                .find(|line| line.starts_with("| `cors_rules` |"))
                .unwrap();
            if !row.contains(&format!("(items: {min}..=100, unique items, ordered)")) {
                failures.push(format!("Markdown lost CorsRules bounds: {row}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r10_normalized_pattern_lengths_reject_empty_intersections_in_both_renderers() {
    let mut failures = Vec::new();
    for (keyword, bound, interval) in [("maxLength", 0, "1 > 0"), ("minLength", 513, "513 > 512")] {
        for in_array in [false, true] {
            let mut property = json!({
                "type":"string", "pattern":r"^[\/]+([^~]*(~[01])*)*{1,512}$",
                keyword:bound
            });
            let pointer = if in_array {
                property = json!({"type":"array", "items":property});
                "/properties/Probe/items"
            } else {
                "/properties/Probe"
            };
            let mut source = cfn_schema_for_codegen_tests("AWS::EC2::VPC").source;
            source["properties"]["Probe"] = property;
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            for (renderer, output) in [
                ("Rust", generate_schema_code(&schema, &schema.type_name)),
                ("Markdown", generate_test_markdown(&schema, &schema.type_name)),
            ] {
                let rejected = output.as_ref().is_err_and(|error| {
                    matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                        Some(schema_layer::SchemaError::Projection { pointer: actual, reason })
                            if actual == pointer
                                && reason.contains("Invalid string length bounds")
                                && reason.contains(interval))
                });
                if !rejected {
                    failures.push(format!(
                        "{renderer} must reject {pointer} ({keyword}={bound}) with a typed projection error: {}",
                        output.err().map_or_else(
                            || "successful emission".into(),
                            |error| format!("{error:#}")
                        )
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
