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

## Git Workflow

### Worktree-Based Development

**IMPORTANT: Use `git wt` (NOT `git worktree`).** `git wt` is a separate tool with its own syntax.

```bash
git wt <branch-name> main    # Create worktree
git wt                       # List worktrees
git wt -d <branch-name>      # Delete worktree (from main worktree)
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
