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

2. Add its DSL resource name and deliberate replacement class to
   `REPLACEMENT_CLASSES` in `codegen.rs`. Codegen fails if this declaration is
   missing.

3. If the resource also needs documentation, add the same type to `scripts/generate-docs.sh`.

4. Run the generation script:

   ```bash
   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh
   ```

5. Verify the generated output:

   ```bash
   cargo test -p carina-provider-awscc
   cargo clippy -p carina-provider-awscc -- -D warnings
   ```

That's it. The Cloud Control API provider handles read/create/update/delete generically.

### Handlerless Cloud Control Types

Codegen normally treats the schema's `handlers` map as the single source for
both Cloud Control support and per-operation IAM permissions. Use
`HANDLERLESS_CLOUD_CONTROL_TYPES` only when a registry schema omits that map
despite the type being provisionable.

Every allowlist entry requires all of the following evidence:

1. `cloudformation describe-type` reports a provisionable `ProvisioningType`
   such as `FULLY_MUTABLE` or `IMMUTABLE`.
2. At least one Cloud Control operation succeeds for the type in the measured
   account and Region.
3. Authoritative AWS documentation identifies non-empty IAM permissions for
   create, read, update, delete, and list. Cite the source URL next to the
   entry, including separate tag operations only when the resource's
   CloudFormation and IAM documentation requires them.

Record the measurement date and Region in the source comment. The typed entry
couples provisionability to its five permission sets, and its const constructor
rejects an empty operation set at compile time. If the registry later publishes
handlers for that type, remove the allowlist entry and use those handlers.

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

# Generate Markdown documentation, including a matching example when present
cargo run --bin codegen -- --file schema.json --type-name AWS::EC2::VPC \
  --format markdown --examples-dir examples
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
| bare `object` (no properties or `patternProperties`) | Must be classified explicitly: `EMPTY_OBJECT_PROPERTIES` emits an empty `Struct`, `STRING_MAP_PROPERTIES` emits `Map<String, String>`, a dedicated `AttributeType` override emits its declared type, and `FREE_FORM_JSON_PROPERTIES` is omitted pending [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144); an unclassified property makes codegen panic with its CloudFormation type and property path |
| `object` with direct-string `patternProperties` | `AttributeType::Map<String, String>` |
| `object` with one `patternProperties` value that `$ref`s an object struct | `AttributeType::Map<String, Struct>` using the referenced definition; every other object-map value shape is rejected |
| `["object","string"]` | `carina_aws_types::iam_policy_document()` only for a typed path listed in `IAM_POLICY_DOCUMENT_PROPERTIES` or `NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES`; properties listed in `FREE_FORM_JSON_PROPERTIES` are omitted pending the JSON-document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144) and documented as not yet supported; every other union makes codegen panic and name the unclassified property |
| `$ref` to a `Tag*` definition | `carina_aws_types::tags_type()` only when the definition is an object with exactly direct-string `Key` and `Value` properties, or has no `properties` and exactly one `patternProperties` entry whose value is a direct string |
| `array` | `AttributeType::unordered_list()` |

CloudFormation `number` properties map to `Float` unless registered as `IntegerFromNumber` in `resource_type_overrides()`. That override maps them to `Int` using the property's CloudFormation `minimum` and `maximum`.

String types are inferred from property names (e.g., `SubnetId` maps to `subnet_id()` type). Resource-specific overrides are defined in `resource_type_overrides()` and `known_string_type_overrides()` within `codegen.rs`.

### Classifying bare objects

Every reachable bare CloudFormation object must use the single `CfnPropertyPath` key builder. The key preserves raw CloudFormation spelling: a top-level property is `Prop`, and a nested property is `<RawParentDefinitionOrInlineProperty>.<Prop>` (for example, `SSESpecification.Marker`, not `SseSpecification.Marker`). Schema generation, definition pre-scan, nested type overrides, cache coverage, and Markdown rendering all use this same key. Unreferenced definitions are not emitted and are skipped by the cache walker. Do not derive a key from a generated Rust struct name or match only the leaf property name.

Inline parents are keyed by their raw property name in the one-level `CfnPropertyPath`; if an inline parent collides with a definition name, the key must become definition-qualified rather than adding another leaf-name heuristic.

Classify each shape in exactly one place. Codegen rejects multiple matching classifications at lookup time, and the table-disjointness test checks IAM, free-form JSON, empty-object, string-map, omitted-property, and dedicated `AttributeType` override keys together:

- `FREE_FORM_JSON_PROPERTIES` contains heterogeneous or recursive JSON documents that Carina cannot represent until [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144) lands. These properties are omitted from generated schemas and regular property tables, then listed in the generated unsupported note.
- `EMPTY_OBJECT_PROPERTIES` contains marker objects whose only meaningful value is `{}`. They generate a closed empty struct, so `{}` validates and any field is an error.
- `STRING_MAP_PROPERTIES` is only for genuine homogeneous string-to-string maps.
- `IAM_POLICY_DOCUMENT_PROPERTIES` explicitly lists every cached path that previously matched the old `*PolicyDocument` suffix rule; `NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES` lists policy paths with other names. Both tables are typed-path and stale-row checked.
- A dedicated `AttributeType` override handles a known non-generic shape, such as the Budgets maps.
- `OMITTED_RESOURCE_PROPERTIES` contains otherwise representable properties deliberately excluded for a documented per-row provider limitation (for example, conflicting read-back or an unavailable AWS capability).

Omission works at both top level and inside generated structs. Generated resource docs collect `FREE_FORM_JSON_PROPERTIES` and `OMITTED_RESOURCE_PROPERTIES` under **Not yet supported**, resolve a reachable nested definition back to its complete snake-case provider/DSL path (for example, `config.child_policy`), and include the table reason. If no provider path is available, the note labels the raw `<Definition>.<Property>` key as a **definition-qualified name**, not as a DSL path. The cache-coverage test fails for both an unclassified bare object and a stale classification row. An unclassified codegen panic points back to this section.

## Schema Cache

Downloaded CloudFormation schemas are cached in `carina-provider-awscc/cfn-schema-cache/` with filenames like `AWS__EC2__VPC.json`. This directory is committed to the repository so codegen is reproducible without AWS access. Use `--refresh-cache` to force re-download.

## Generating Documentation

```bash
aws-vault exec <profile> -- ./scripts/generate-docs.sh
```

Output goes to `generated-docs/awscc/`. Each resource gets a Markdown file with
Starlight/Astro frontmatter. Codegen inserts example code from
`examples/{resource}/main.crn` before validating the page, when that file is
present; the shell script only adds frontmatter and removes the generated H1.

Struct and enum section headings use the DSL field path. A `$ref` definition
reached from more than one path instead uses its `snake_case` definition name,
includes a `Used by` line, and lends that heading prefix to nested sections;
`PageBuilder` resolves any resulting slug collisions.

`carina-docgen` allocates registry-compatible heading ids when headings are
written, resolves enum and struct links from typed targets when the page is
finished, and checks the finished Markdown. Generation fails on a missing
target, a dead local fragment, a Markdown link hidden inside a code span, or a
duplicated enum/struct section heading. The docs-bundle CI job runs the same
checker over the shipped tarball after its pages have had their H1 headings
reconstructed from frontmatter.
