# Schema Code Generation

This document explains how to generate and maintain resource schemas from CloudFormation resource type schemas.

## Overview

The AWSCC provider uses a generic Cloud Control API implementation, so adding a new resource only requires generating its schema definition. No service-specific code is needed.

The codegen pipeline:

1. Downloads CloudFormation resource type schemas from the AWS API
2. Generates Rust schema code (`schemas/generated/`) via the `codegen` binary
3. Optionally generates Markdown documentation (`generated-docs/`)

## Prerequisites

- AWS credentials accessible via `aws-vault` (needed to call `aws cloudformation describe-type`)
- Rust toolchain (to build the `codegen` binary)

## Generating Schemas

From the project root:

```bash
aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh
```

This will:

1. Build the `codegen` binary
2. Download CloudFormation schemas for all configured resource types (cached in `carina-provider-awscc/cfn-schema-cache/`)
3. Generate Rust files in `carina-provider-awscc/src/schemas/generated/`
4. Generate per-service `mod.rs` and top-level `mod.rs`
5. Run `cargo fmt`

To force re-download of all schemas (e.g., after an AWS API update):

```bash
aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh --refresh-cache
```

## Adding a New Resource

1. Add the CloudFormation type name to the `RESOURCE_TYPES` array in `carina-provider-awscc/scripts/generate-schemas.sh`:

   ```bash
   RESOURCE_TYPES=(
       # ... existing types ...
       "AWS::Organizations::Account"
   )
   ```

2. If the resource also needs documentation, add the same type to `scripts/generate-docs.sh`.

3. Run the generation script:

   ```bash
   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh
   ```

4. Verify the generated output:

   ```bash
   cargo test -p carina-provider-awscc
   cargo clippy -p carina-provider-awscc -- -D warnings
   ```

That's it. The Cloud Control API provider handles read/create/update/delete generically.

## Generated File Structure

```
carina-provider-awscc/src/schemas/generated/
├── mod.rs                          # Top-level: schema configs, enum lookups
├── awscc_types.rs                  # Shared type definitions (not generated)
├── ec2/
│   ├── mod.rs                      # Module declarations
│   ├── vpc.rs                      # AWS::EC2::VPC schema
│   ├── subnet.rs
│   └── ...
├── organizations/
│   ├── mod.rs
│   ├── organization.rs
│   └── account.rs
└── ...
```

Each resource file contains:

- **Enum constant arrays** for validation (e.g., `VALID_INSTANCE_TENANCY`)
- **Validator functions** for range/pattern constraints
- **`{resource}_config()`** returning `AwsccSchemaConfig` with the full schema
- **`enum_valid_values()`** for enum normalization during read-back
- **`enum_alias_reverse()`** for DSL alias to AWS value mapping

The top-level `mod.rs` builds:

- `SCHEMA_CONFIGS` — all schema configs (lazy-initialized)
- `SCHEMA_CONFIG_INDEX` — O(1) lookup by resource type name
- `ENUM_VALID_VALUES` — enum validation lookup
- `ENUM_ALIAS_DISPATCH` — per-resource enum alias reverse functions

## Codegen Binary

The `codegen` binary (`carina-provider-awscc/src/bin/codegen.rs`) can also be used directly:

```bash
# From stdin
aws cloudformation describe-type \
  --type RESOURCE --type-name AWS::EC2::VPC \
  --query 'Schema' --output text | \
  cargo run --bin codegen -- --type-name AWS::EC2::VPC

# From cached file
cargo run --bin codegen -- --file cfn-schema-cache/AWS__EC2__VPC.json --type-name AWS::EC2::VPC

# Print resource names (useful for scripting)
cargo run --bin codegen -- --type-name AWS::EC2::VPC --print-module-name       # vpc
cargo run --bin codegen -- --type-name AWS::EC2::VPC --print-full-resource-name # ec2_vpc
cargo run --bin codegen -- --type-name AWS::EC2::VPC --print-dsl-resource-name  # ec2.vpc

# Generate Markdown documentation
cargo run --bin codegen -- --file schema.json --type-name AWS::EC2::VPC --format markdown
```

## Type Mapping

| CloudFormation | Carina Type |
|---|---|
| `string` | `AttributeType::String` (or specialized: `ipv4_cidr()`, `arn()`, etc.) |
| `string` with `enum` | `AttributeType::StringEnum` |
| `integer` | `AttributeType::Int` (with range validator if constrained) |
| `boolean` | `AttributeType::Bool` |
| `number` | `AttributeType::Float` |
| `object` (properties) | `AttributeType::Struct` with `StructField` |
| `array` | `AttributeType::unordered_list()` |

String types are inferred from property names (e.g., `SubnetId` maps to `subnet_id()` type). Resource-specific overrides are defined in `resource_type_overrides()` and `known_string_type_overrides()` within `codegen.rs`.

## Schema Cache

Downloaded CloudFormation schemas are cached in `carina-provider-awscc/cfn-schema-cache/` with filenames like `AWS__EC2__VPC.json`. This directory is gitignored. Use `--refresh-cache` to force re-download.

## Generating Documentation

```bash
aws-vault exec <profile> -- ./scripts/generate-docs.sh
```

Output goes to `generated-docs/awscc/`. Each resource gets a Markdown file with Starlight/Astro frontmatter. Example code from `examples/{resource}/main.crn` is automatically inserted if present.
