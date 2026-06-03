# CLAUDE.md

This file provides guidance to Claude Code when working with the carina-provider-awscc repository.

## Repository Overview

This is the AWS Cloud Control provider for [Carina](https://github.com/carina-rs/carina), split out as a standalone repository. It uses the AWS Cloud Control API to manage resources via CloudFormation resource type schemas. It depends on carina-core, carina-aws-types, carina-plugin-sdk, and carina-provider-protocol via git dependencies from the main carina repository.

## Build and Test Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Build WASM target
cargo build -p carina-provider-awscc --target wasm32-wasip2 --release

# Run clippy
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

### With AWS Credentials

```bash
aws-vault exec <profile> -- cargo test
```

## Crate Structure

- **carina-provider-awscc**: The AWSCC provider implementation. Includes a `codegen` binary for generating resource definitions from CloudFormation schemas. Builds as both a native binary and a WASM component.

## Dependencies on carina (main repo)

This repository depends on crates from `github.com/carina-rs/carina`:
- `carina-core` — Core types, parser, traits
- `carina-aws-types` — AWS-specific type definitions
- `carina-plugin-sdk` — Plugin SDK for building providers
- `carina-provider-protocol` — Protocol definitions for provider communication

These are specified as `git` dependencies in `Cargo.toml`. For local development, you can override them in `.cargo/config.toml`:

```toml
[patch."https://github.com/carina-rs/carina"]
carina-core = { path = "../carina/carina-core" }
carina-aws-types = { path = "../carina/carina-aws-types" }
carina-plugin-sdk = { path = "../carina/carina-plugin-sdk" }
carina-provider-protocol = { path = "../carina/carina-provider-protocol" }
```

## Code Generation

Resource definitions are generated from CloudFormation resource type schemas:

```bash
cargo run --bin codegen -- <schema-path>
```

## Adding a Resource: Required Tests

When adding a new resource (see `CODEGEN.md` for the codegen steps), two
testing obligations apply on top of the generated schema:

### 1. Add a winterbaume CloudControl round-trip test

Every new resource must get an integration test under
`carina-provider-awscc/tests/<service>_<resource>_cloudcontrol_roundtrip.rs`
that drives the real `Provider` trait (`Provider::create` then
`Provider::read`) against an in-process winterbaume CloudControl mock
(`winterbaume_core::MockAws` + `winterbaume_cloudcontrol::CloudControlService`).
Mirror the existing `dynamodb_table_cloudcontrol_roundtrip.rs`.

The mock returns the desired (create) state **verbatim**. So the test
proves only what the mock can prove: that create -> read *wiring*
round-trips the resource's **structured** fields through CloudControl
serialization — list-of-string and list-of-struct fields survive as
structured `List`/`Map` values instead of being flattened or
stringified. Scope the assertions to that. Do **not** assert full-field
equality as if every create field comes back unchanged on real AWS — that
is a mock artifact, not real AWS behaviour (see below). State this
limitation in the test's module doc comment.

### 2. Reconcile mock behaviour with real AWS, and file a winterbaume issue **per resource**

The generic winterbaume mock does **not** reproduce real AWS CloudControl
behaviours that the CloudFormation resource-type schema drives:
write-only field stripping, read-only field synthesis (e.g. `Arn`),
schema-default fill-in, and value normalization. For each new resource,
verify the actual AWS behaviour by creating it on a live account and
capturing the real `GetResource` output, then compare it to what the mock
returns.

File a **separate winterbaume issue for each new resource** — do **not**
fold it into a single umbrella issue. Even though every case shares the
same root cause (the CloudControl layer returns the create-time
`DesiredState` verbatim without consulting any CFN schema), *which*
read-only and default properties real AWS fills in is **different for
every resource type**, so the fix can only be verified against a
concrete, resource-specific `DesiredState → real GetResource` diff. This
is the precedent the existing reports already follow: winterbaume #6
(`AWS::KMS::Key`) and #7 (`AWS::DynamoDB::Table`) are separate issues for
the same root cause, each carrying its own captured diff. A new resource
gets its own issue, cross-linked to the existing ones (e.g. "same root
cause as #6 / #7"), not a comment on them.

When filing, follow winterbaume's own agent skill —
`skills/winterbaume-bug/SKILL.md` in that repo — **verbatim**. It mandates
a fixed `### Affected AWS service / Summary / Reproduction / Expected /
Actual / version` layout that an auto-label GitHub Action parses; a
free-form body silently breaks labelling. The skill also requires a real,
actually-executed reproduction (run it and paste the output — never invent
one), a reported duplicate search, and `git rev-parse HEAD` for the
version field. winterbaume is **not** a `carina-rs` repository, so the
external-repo rule in `CLAUDE.local.md` also applies: draft the issue per
that skill, then get explicit confirmation before `gh issue create`. The
AWS-specific behaviours themselves must be verified against live AWS, not
asserted in the mock round-trip test.

## Git Workflow

### Worktree-Based Development

```bash
git worktree add .worktrees/<branch-name> -b <branch-name> main   # Create worktree
git worktree list                                                  # List worktrees
git worktree remove .worktrees/<branch-name>                       # Delete worktree (from the main worktree)
```

### Submodule Initialization

This repo uses a git submodule for `carina-plugin-wit/`. After `git pull` or creating a new worktree, initialize the submodule:

```bash
git submodule update --init --recursive
```

Without this, builds will fail because `wit_bindgen::generate!` cannot find the WIT files.

## Code Style

- **Commit messages**: Write in English
- **Code comments**: Write in English
