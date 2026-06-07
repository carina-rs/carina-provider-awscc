# CLAUDE.md

This file provides guidance to Claude Code when working with the carina-provider-awscc repository.

## Repository Overview

This is the AWS Cloud Control provider for [Carina](https://github.com/carina-rs/carina), split out as a standalone repository. It uses the AWS Cloud Control API to manage resources via CloudFormation resource type schemas. It depends on carina-core, carina-plugin-sdk, and carina-provider-protocol via git dependencies from the main carina repository. `carina-aws-types` lives in this repository (a local copy, not shared from the main repo).

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
- **carina-aws-types**: AWS-specific type definitions. A local copy lives in this repo (the same crate is duplicated in `carina-provider-aws`; it is not shared from the main carina repository).

## Dependencies on carina (main repo)

This repository depends on crates from `github.com/carina-rs/carina`:
- `carina-core` — Core types, parser, traits
- `carina-plugin-sdk` — Plugin SDK for building providers
- `carina-provider-protocol` — Protocol definitions for provider communication

These are specified as `git` dependencies in `Cargo.toml`. For local development, you can override them in `.cargo/config.toml`:

```toml
[patch."https://github.com/carina-rs/carina"]
carina-core = { path = "../carina/carina-core" }
carina-plugin-sdk = { path = "../carina/carina-plugin-sdk" }
carina-provider-protocol = { path = "../carina/carina-provider-protocol" }
```

`carina-aws-types` is **not** a main-repo dependency — it is a local crate in
this repository (`carina-provider-awscc/Cargo.toml` references it as
`{ path = "../carina-aws-types" }`), so it needs no patch entry.

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

winterbaume-cloudcontrol shapes the `GetResource` read model **per registered
CloudFormation resource type**. As of winterbaume-cloudcontrol 1.0.1, the
registered set is exactly:

- `AWS::KMS::Key` - winterbaume #6, closed/shaped
- `AWS::DynamoDB::Table` - winterbaume #7, closed/shaped
- `AWS::ECS::Cluster` - winterbaume #8, closed/shaped
- `AWS::ElasticLoadBalancingV2::TargetGroup` - winterbaume #9, closed/shaped
- `AWS::ElasticLoadBalancingV2::LoadBalancer` - winterbaume #10, closed/shaped
- `AWS::ElasticLoadBalancingV2::Listener` - winterbaume #11, closed/shaped

Registered types reproduce the real AWS CloudControl schema shaping:
write-only field stripping, read-only field synthesis (for example `Arn`), and
schema-default fill-in. Unregistered types fall back to returning the
create-time `DesiredState` verbatim, which is the old mock behaviour.

The round-trip test's assertions therefore depend on whether the resource is in
that shaping registry.

For a **registered** resource, assert the full CFN-schema-shaped read state
exhaustively. Build the expected attribute map from the real `GetResource`
shape and compare the whole map so missing keys, extra keys, wrong defaults,
and incorrectly preserved write-only fields all fail. Use
`kms_key_cloudcontrol_roundtrip.rs` (including its generated-UUID `arn` /
`key_id` handling) and `ecs_cluster_cloudcontrol_roundtrip.rs` as the
templates. To prove the test really exercises shaping, and is not passing by
accident, it must fail against the pre-shaping verbatim mock and pass against
the shaping mock.

For an **unregistered** resource, the mock still returns the create-time
`DesiredState` verbatim. Assert only structural preservation: list-of-string
and list-of-struct fields survive as structured `List`/`Map` values instead of
being flattened or stringified. Do **not** assert full shaped equality for an
unregistered resource, because that would lock in a mock artifact rather than
real AWS behaviour. State this limitation in the test's module doc comment.
Use this pattern only until the resource gets a winterbaume shaper; after that,
upgrade the test to full shaped equality.

### 2. For unregistered resources, reconcile with real AWS and file a winterbaume issue **per resource**

For an unregistered new resource, verify the actual AWS behaviour by creating
it on a live account, capturing the real `GetResource` output, and comparing
that to the mock's verbatim read state. Then file a **separate winterbaume issue
for each resource** so the resource can get a shaper. Do **not** fold multiple
resources into a single umbrella issue. Which read-only and default properties
real AWS fills in differs per resource type, so each fix needs its own captured
`DesiredState` -> real `GetResource` diff. Cross-link to the existing examples:
#6 (`AWS::KMS::Key`), #7 (`AWS::DynamoDB::Table`), #8
(`AWS::ECS::Cluster`), #9 (`AWS::ElasticLoadBalancingV2::TargetGroup`), #10
(`AWS::ElasticLoadBalancingV2::LoadBalancer`), and #11
(`AWS::ElasticLoadBalancingV2::Listener`) are closed/shaped.

When filing, follow winterbaume's own agent skill,
`skills/winterbaume-bug/SKILL.md` in that repo, **verbatim**. It mandates a
fixed `### Affected AWS service / Summary / Reproduction / Expected / Actual /
version` layout that an auto-label GitHub Action parses; a free-form body
silently breaks labelling. The skill also requires a real, actually-executed
reproduction (run it and paste the output; never invent one), a reported
duplicate search, and `git rev-parse HEAD` for the version field. winterbaume
is **not** a `carina-rs` repository, so the external-repo rule in
`CLAUDE.local.md` also applies: draft the issue per that skill, then get
explicit confirmation before `gh issue create`.

Once winterbaume registers the resource and releases it, upgrade that
resource's round-trip test from structural-only assertions to full shaped
equality. The dependency bump plus the KMS, DynamoDB, and ECS test
strengthening in the winterbaume 1.0.0 work is the model for that upgrade.

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
