# Trailing Dot Normalization for Route 53 DNS Names

## Goal

Prevent spurious replacement diffs when Route 53 returns DNS names with a trailing dot (FQDN format) that the user omitted in their `.crn` file.

## Problem

Route 53 always returns fully qualified domain names with a trailing dot (`carina-rs.dev.`). When the user writes `name = 'carina-rs.dev'`, the differ sees a change on a `create_only` attribute and schedules a replacement — even though the two forms are DNS-equivalent.

## Chosen Approach

**Extend `aws_value_to_dsl` to apply `to_dsl` on non-namespaced `Custom` types.**

The `to_dsl` field already exists on `AttributeType::Custom` but is currently only used for namespaced enums. By also checking it for plain Custom types in the `aws_value_to_dsl` conversion, any Custom attribute can declare a read-path normalization function.

This is generic: future cases where AWS APIs normalize values differently from user input (e.g., trailing slashes, case normalization) can use the same mechanism.

## Design

### 1. `aws_value_to_dsl` change (conversion.rs)

Add a check before the final `json_to_value` fallback:

```rust
// For Custom types with to_dsl, apply the transformation on read.
if let AttributeType::Custom { to_dsl: Some(transform), .. } = attr_type
    && let Some(s) = value.as_str()
{
    return Some(Value::String(transform(s)));
}
```

This handles non-namespaced Custom types. Namespaced Custom types are already handled earlier in the function.

### 2. Schema change (hosted_zone.rs)

Add `to_dsl` to the `name` attribute's Custom type:

```rust
to_dsl: Some(|s: &str| s.strip_suffix('.').unwrap_or(s).to_string()),
```

### 3. Codegen override (codegen.rs)

Add a new `TypeOverride` variant or a `to_dsl` override table so the codegen can emit `to_dsl` for specific attributes. This ensures re-generation preserves the fix.

### 4. aws provider (record_set.rs)

The `aws.route53.record_set` resource is hand-coded (not codegen). Its `name` attribute also needs trailing dot stripping, but that's handled differently since it uses `normalize_dns_name()` in the write path. The read path (`extract_attributes`) strips the trailing dot already. Verify this is consistent.

## Scope

| Repository | Resource | Attribute | Change |
|-----------|----------|-----------|--------|
| carina-provider-awscc | route53.hosted_zone | name | Add `to_dsl` to strip trailing dot |
| carina-provider-awscc | (conversion.rs) | - | Handle Custom `to_dsl` in `aws_value_to_dsl` |
| carina-provider-awscc | (codegen.rs) | - | Add `to_dsl` override support |
| carina-provider-aws | route53.record_set | name | Verify read path already handles this |

## Edge Cases

- User writes `name = 'carina-rs.dev.'` (with dot): `to_dsl` strips it → `carina-rs.dev`. Consistent.
- Empty string: `strip_suffix('.')` returns the original. Safe.
- Multiple trailing dots (malformed): only one dot stripped. AWS would reject this anyway.

## No inverse needed

`dsl_value_to_aws` does not need a corresponding change. AWS Route 53 accepts both `carina-rs.dev` and `carina-rs.dev.` — the trailing dot is optional on input.
