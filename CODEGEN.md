# Schema Code Generation

This document explains how to generate and maintain resource schemas from CloudFormation resource type schemas.

## Overview

The AWSCC provider uses a generic Cloud Control API implementation, so adding a new resource only requires generating its schema definition. No service-specific code is needed.

The codegen pipeline:

1. Downloads CloudFormation resource type schemas from the AWS API
2. Converts JSON into the typed Layer, then validates shared projections
3. Generates Rust schema code (`schemas/generated/`) via the `codegen` binary
4. Optionally generates Markdown documentation (`generated-docs/`) from the same projections

### Layer and projection boundary

[`schema_layer.rs`](carina-provider-awscc/src/bin/codegen/schema_layer.rs) is the
production JSON conversion boundary. Its closed keyword vocabulary retains
schema nodes, annotations, resource and handler metadata, reference targets,
every composition arm, both dependency forms, and `contains`. Conversion uses
exhaustive matches; unsupported names and unreadable values produce typed
SchemaError diagnostics with source pointers. Consumers query this immutable
Layer rather than interpreting schema JSON again. Raw fixture editing is test-only.

[`schema_projection.rs`](carina-provider-awscc/src/bin/codegen/schema_projection.rs)
provides the shared constraints, shape and provenance used by Rust and Markdown.
References and `allOf` intersect constraints; defaults and descriptions remain
site annotations, so a reference-site default can differ from its target's default.
Equivalent array alternatives retain their children and bounds. Unsupported
alternatives and bounds that cannot be represented exactly return typed projection
errors. Resource compositions can supply requirements and field groups; property
contributions requiring merged root provenance currently return a typed error
before either renderer can emit only the direct root property.

[`tag_shape.rs`](carina-provider-awscc/src/bin/codegen/tag_shape.rs) caches one
classify_tag_shape result per Layer node. Tag admission, candidate evidence,
DSL naming, default_tags support, wire metadata, guards and both renderers consume
this TagShape judgement. Its admitted map contains the resolved string leaves,
collection bounds and source provenance. Rendering does not walk tag schemas again.

## SDK Access

The codegen binary and generated schemas do not construct SDK clients. The
provider uses guarded handles from `carina_aws_types::sdk` for CloudControl,
S3, STS, and KMS, and `GuardedAssumeRoleProvider` for role credentials. Service
handles accept the bounded shared configuration and the idempotency lookup in
[`resend_safety.rs`](carina-provider-awscc/src/resend_safety.rs); borrow the SDK
client through `.inner()` for operations. CloudControl retains its existing
Smithy-model-based idempotency table.

Enable service features on the normal `carina-aws-types` dependency. Its workspace
dependency disables default features; the dev-dependency adds only `test-utils`.
Use its `sdk::<service>::{error, operation, primitives, types}` re-exports in both
provider code and tests. Direct `aws-sdk-*` and `aws-config` dependencies are not
needed; shared configuration comes from `aws-types`. The upstream facade has
compile-fail doctests for inaccessible raw client and configuration types.

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
cargo run --bin codegen -- --type-name AWS::EC2::VPC --print-dsl-resource-name  # ec2.Vpc

# Generate Markdown documentation, including the required matching example
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
| Declared resource tag property | `AttributeType::map_with_key(key, value)` for an object with exactly case-sensitive string `Key` and `Value` fields, or a closed map with one string-valued `patternProperties` entry. Local references, aliases and `allOf` may supply the shape or leaf constraints; every non-null union arm must agree. Leaves retain `minLength`, `maxLength` and approved `pattern` constraints; explicit leaf `enum` or `const` constraints currently produce a typed error. Pattern-map keys use their regex. Array `minItems` and `maxItems` become map entry bounds through `AttributeType::refined_map`; without effective bounds the map stays unbounded. See [String patterns](#string-patterns) |
| `array` | `AttributeType::unordered_list()` |

CloudFormation `number` properties map to `Float`; `integer` properties map to `Int`. A narrowing preserves the CFN kind, including through numeric ranges and array-item refinements. Numeric endpoints must be exactly representable by the emitted type; otherwise both generators return a typed `SchemaError`.

String types are inferred from property names (e.g., `SubnetId` maps to `subnet_id()` type). Resource-specific overrides are defined in `resource_type_overrides()` and `known_string_type_overrides()` within `codegen.rs`.

An identity names a type while retaining the field's effective CFN constraints:

```rust
m.insert(
    ("AWS::EC2::Subnet", "VpcId"),
    TypeOverride::identity(Some(StringIdentity::Ec2VpcId)),
);
```

A structural restriction requires a `NarrowingKind` and checked `AwsEvidence`. Documentation evidence includes an AWS URL, a verbatim quotation that supports the restriction, and a checked date. For example:

```rust
let evidence = AwsEvidence::new(
    "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters",
    "Length Constraints: Minimum length of 1. Maximum length of 64.",
    CheckedOn { year: 2026, month: 11, day: 7 },
)?;
m.insert(
    ("AWS::IAM::Role", "RoleName"),
    TypeOverride::identity(None).with_narrowing(
        NarrowingKind::Primitive {
            expression: ConstraintSpec::string(
                None, Some((Some(1), Some(64))), PatternLiteral::Quoted,
            ),
            display: OverrideTypeDisplay::new("String(len: 1..=64)"),
            block_name: None,
            default: None,
        },
        evidence,
    ),
);
```

For recorded service behavior, use `AwsEvidence::observed` with exact inputs, results, verified directions, region, date, method, and a source record. `insert_evidenced` looks up the reviewed row in `narrowing_evidence.json`; a candidate without a review cannot become a narrowing. Both renderers validate the intersection with CFN through `TypeOverride::validate_source_shape` and `emission_plan::lower`. Evidence does not permit changing the CFN kind or discarding constraints.

Tag-map count overrides use `NarrowingKind::Array`, keyed by the resource type and raw CFN property path (`Tags`):

```rust
m.insert_evidenced(
    ("AWS::Scheduler::ScheduleGroup", "Tags"),
    NarrowingKind::Array {
        items: None,
        min_items: None,
        max_items: Some(50),
    },
);
```

This example requires the existing ScheduleGroup observation in `narrowing_evidence.json`. It retains the derived key/value types and tightens CFN's `maxItems: 200` to the observed service limit of 50. Both bounds reach Rust and Markdown; an implicit lower bound of zero uses the unconstrained length normal form. Generated docs display map counts as `(entries: ..=50)` or `(entries: 2..=5)`, using the same formatter as bounded lists.

Resource tagging classification uses `resource_type_overrides().classify_tagging((cfn_type, "/properties/PropertyName"), TaggingOverride::TagProperty)`. These keys are raw CFN JSON pointers. Use this declaration when CFN omits its `tagging` block; otherwise `tagging.tagProperty` identifies the property (defaulting to `/properties/Tags` for a taggable resource). A top-level property named `Tags` is also recognized without metadata. All three sources feed the same tag-map projection, constraints, `TagsProperty` wire metadata, portable `TagsKeyValueCheck` declaration, and documentation.

Defaults are site annotations: a referenced definition's default does not become a default on an unannotated reference. Optional resource attributes support scalar, list, map, and struct defaults; struct literals translate CFN field names to DSL names while map keys stay unchanged. Nested field defaults remain in generated Rust descriptions and Markdown with an explicit server-default diagnostic because the pinned core has no `StructField` default API. Required/output defaults and JSON defaults whose shape differs from the emitted DSL type are also documented as server defaults. Null, unrepresentable numbers, defaults on collection elements without a field site, and defaults on CFN Key/Value tag shapes transformed into DSL maps produce typed errors.

The identified tag property always has the DSL name `tags`. Generated attributes retain the CFN name through `with_provider_name("ResourceTags")` or `with_provider_name("HostedZoneTags")`, and tag wire metadata uses that same CFN name. Both generators fail if multiple distinct tag properties are identified or if renaming the tag property would collide with another generated DSL attribute named `tags`. Multiple sources may identify the same CFN property.

Both generators reject an unclassified top-level property when cached TagShape evidence contains both `key` and `value` in any ASCII case after an `items` or `contains` edge, including extra fields or non-string values. The classifier reads typed Layer edges through local references, `allOf`, every `oneOf`/`anyOf` arm, `items`, `contains` and schema dependencies; property-list dependencies also contribute required-name evidence. It collects evidence across resource composition source sites and guards cycles. Names split between alternative arms still require classification, but do not establish an admissible tag shape. Candidate detection excludes intermediate child scopes under `properties`, `patternProperties` and `additionalProperties`. Conditional dependencies and `contains` cannot become unconditional map constraints. A deliberately non-tag list requires `TaggingOverride::NonTagList { reason: "..." }`; a declared `TagProperty` must pass the cached admission judgement or both generators fail with a typed error and source path.

Only top-level properties can be a resource's tag attribute: carina-core merges `default_tags` and runs `TagsKeyValueCheck` on the top-level `tags` only. Nested Key/Value arrays such as S3 `TagFilters` are outside this guard.

The 2026-11-06 survey covered all 55 cached schemas and found 41 top-level tag-shaped arrays. Six were previously unrecognized:

| CFN resource | Raw CFN path | Classification and CFN documentation |
| --- | --- | --- |
| `AWS::Budgets::Budget` | `/properties/ResourceTags` | [Resource tags](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-budgets-budget.html#cfn-budgets-budget-resourcetags); explicit tag-property declaration supplies the missing CFN metadata. |
| `AWS::Route53::HostedZone` | `/properties/HostedZoneTags` | [Resource tags](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-route53-hostedzone.html#cfn-route53-hostedzone-hostedzonetags); explicit declaration agrees with CFN metadata, which codegen previously ignored because the property was not named `Tags`. |
| `AWS::EC2::IPAMPool` | `/properties/AllocationResourceTags` | [Allocation requirements](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-ipampool.html#cfn-ec2-ipampool-allocationresourcetags); deliberately non-tag list. The pool's own tags are `/properties/Tags`. |
| `AWS::ElasticLoadBalancingV2::Listener` | `/properties/ListenerAttributes` | [Listener settings](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-elasticloadbalancingv2-listener-listenerattribute.html); deliberately non-tag list. |
| `AWS::ElasticLoadBalancingV2::LoadBalancer` | `/properties/LoadBalancerAttributes` | [Load balancer settings](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-elasticloadbalancingv2-loadbalancer-loadbalancerattribute.html); deliberately non-tag list. |
| `AWS::ElasticLoadBalancingV2::TargetGroup` | `/properties/TargetGroupAttributes` | [Target group settings](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-elasticloadbalancingv2-targetgroup-targetgroupattribute.html); deliberately non-tag list. |

Budget and HostedZone use flat map syntax (`tags = { Project = 'carina-registry' }`), replacing lists of `key`/`value` structs or repeated tag blocks. Their examples, acceptance fixtures, and generated docs use this shape. The cached Budget schema has no `maxItems`; codegen does not invent a count bound from external documentation.

The DSL name `tags` is carina-core's contract for merging `default_tags` and executing `TagsKeyValueCheck`. Enforcing this name in codegen makes Budget and HostedZone compatible with the pinned core revision (`4311557`) without a core or protocol extension. Provider create, read, and update paths use the existing CFN provider-name mapping.

### String patterns

Every emitted AWSCC string pattern must match the whole value, except for an exact registered substring exception. The shared `schemas::pattern::ApprovedPattern` has a private field: `whole(source)` preserves an already whole-anchored source and otherwise emits `^(?:source)\z`. A whole-anchored source starts with `^`, ends with `$` or `\z`, and anchors every alternative: `^A$|^B$` qualifies; `^A|B$` does not. The predicate parses the regex so escaped anchors, character classes and multiline flags cannot bypass the rule. Rust's `$` without multiline mode means end of text. Invalid syntax is retained for diagnostics, never dropped.

CFN string refinements share `string_constraints()` between Rust and Markdown. Override expressions use typed string refinements, and runtime constructors in `catalog.rs` and `awscc_types.rs` use adapters accepting `ApprovedPattern`. Codegen compiles normalized patterns in Carina's schema dialect and rejects an invalid or unapproved pattern with its resource type, full JSON source pointer and regex. The recursive `all_registered_provider_schema_patterns_are_whole_anchored_or_registered_exceptions` test audits the actual provider schemas, including definitions and collection, struct and union members.

`STRING_PATTERN_REWRITES` pins resource types, full source pointers and exact source regexes. Preserve constraints when removing unsupported look-around, including ECR `RepositoryName`'s length intersection. Logs maps `\Z` to strict `\z`; that is the intended reading, not ECMAScript equivalence (`\Z` means literal `Z` there). WAFv2's 17 `.*\S.*` sites use `^[\s\S]*\S[\s\S]*\z`: both languages accept exactly values containing a `\S` character, including newlines around it. Generic wrapping would change that newline behavior. Use ASCII `[0-9]` and `[0-9A-Za-z_]` when expanding ECMAScript `\d` and `\w` in rewrites. Add accept and reject cases when changing a rewrite.

`SUBSTRING_PATTERN_EXCEPTIONS` contains only Scheduler `EventBridgeParameters.Source`, keyed by resource type, full source pointer and exact source regex, with its reviewed emitted form and compatibility reason. It preserves the existing search language because CFN and the API publish differently anchored versions. Before either CLI output mode, codegen rejects changed or missing rewrite and exception sources, including sources replaced by runtime constructors. The cached-schema compilation test also checks registry coverage. Review, re-key or delete stale rows instead of silently inheriting an old decision.

### Required field groups

[`field_groups.rs`](carina-provider-awscc/src/bin/codegen/field_groups.rs) projects
`oneOf` requirements into exactly-one groups and `anyOf` requirements into
at-least-one groups. Common requirements stay unconditional; unsupported
multi-field alternatives return typed errors rather than flattening their members.
Resource scopes use FieldGroup::exactly_one or FieldGroup::at_least_one; nested
structs use AttributeType::struct_with_field_groups at every inline, named,
list-item and map-value use. The containing optional object can remain absent;
when present, its group is enforced, including for false-valued boolean members.

Group admission checks emitted siblings after omission and deprecation filtering.
Members must be optional inputs without defaults; provider-populated, required,
defaulted or missing members produce typed projection errors. Reference reuse
compares descendant constraints as well as immediate requirements, so a local
child group cannot alter another use of the definition. Description-derived
resource groups supplement only schema-disjoint members: overlapping schema
declarations take precedence. Both renderers consume the shared schema groups
and Markdown describes each rule beside its containing struct.

### Classifying bare objects

Every reachable bare CloudFormation object must use the single `CfnPropertyPath` key builder. The key preserves raw CloudFormation spelling: a top-level property is `Prop`, and a nested property is `<RawParentDefinitionOrInlineProperty>.<Prop>` (for example, `SSESpecification.Marker`, not `SseSpecification.Marker`). Schema generation, definition pre-scan, nested type overrides, cache coverage, and Markdown rendering all use this same key. Unreferenced definitions are not emitted and are skipped by the cache walker. Do not derive a key from a generated Rust struct name or match only the leaf property name.

Inline parents are keyed by their raw property name in the one-level `CfnPropertyPath`; if an inline parent collides with a definition name, the key must become definition-qualified rather than adding another leaf-name heuristic.

Classify each shape in exactly one place. Codegen rejects multiple matching classifications at lookup time, and the table-disjointness test checks IAM, free-form JSON, empty-object, string-map, omitted-property, and dedicated `AttributeType` override keys together:

- `FREE_FORM_JSON_PROPERTIES` contains heterogeneous or recursive JSON documents that Carina cannot represent until [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144) lands. These properties are omitted from generated schemas and regular property tables, then listed in the generated unsupported note.
- `EMPTY_OBJECT_PROPERTIES` contains candidates for marker objects represented by `{}`. Each row registers an evidenced `NarrowingKind::Primitive` with a typed empty-object constraint; common source-shape validation must succeed before either renderer consumes it. Generated docs include its AWS evidence. This supported subset generates a closed empty struct, so `{}` validates and any field is an error.
- `STRING_MAP_PROPERTIES` is only for genuine homogeneous string-to-string maps and uses the same evidenced narrowing boundary. Neither classification table authorizes emission by itself.
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
`examples/{service}_{resource}/main.crn` before validating the page. This file
is required for every documented resource and must declare that documented
`awscc.<service>.<Type>` resource. The Markdown preprocessor drops every
top-level comment (indented or not, using `#` or `//`), the unnamed provider
block, and blank lines at both ends. Put reader-facing notes in indented comments
inside a resource body so they are retained. The docs build fails when an
example is missing or does not declare the documented resource; the shell script
only adds frontmatter and removes the generated H1.

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
