# Implementation Plan: Trailing Dot Normalization

## Task 1/3: Apply Custom to_dsl in aws_value_to_dsl

**Files:**
- `carina-provider-awscc/src/provider/conversion.rs`

**Changes:**
- Add a Custom type check in `aws_value_to_dsl` before the `json_to_value` fallback
- When `AttributeType::Custom { to_dsl: Some(transform), namespace: None, .. }`, apply `transform` to string values

**Tests:**
- Unit test in conversion.rs: Custom type with `to_dsl` transforms value on read
- Unit test: Custom type without `to_dsl` passes through unchanged
- Unit test: Custom type with namespace (existing enum behavior) is unaffected

## Task 2/3: Add to_dsl override support in codegen

**Files:**
- `carina-provider-awscc/src/bin/codegen.rs`

**Changes:**
- Add a `to_dsl` override mechanism (either a new `TypeOverride` variant or a separate override table keyed by `(resource_type, property_name)`)
- For `(AWS::Route53::HostedZone, Name)`, emit `to_dsl: Some(|s: &str| s.strip_suffix('.').unwrap_or(s).to_string())`
- Regenerate `hosted_zone.rs`

**Tests:**
- Codegen test: verify the override produces correct `to_dsl` in generated code
- Regenerate and verify `hosted_zone.rs` has the `to_dsl` field

## Task 3/3: Verify aws provider record_set read path

**Files:**
- `carina-provider-aws/src/services/route53/record_set.rs`

**Changes:**
- Verify `extract_attributes` already strips trailing dot from `name`
- If not, add `strip_suffix('.')` in the read path
- This is a hand-coded file, so direct edit is fine

**Tests:**
- Unit test or manual verification that read values don't have trailing dots
