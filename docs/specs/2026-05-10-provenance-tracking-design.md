# Attribute Provenance Tracking — Design

<!-- derived-from ./specs/open-issue.md -->

## Goal

Eliminate spurious `- field: "value"` lines in `carina plan` output that
arise when AWS (and other providers) return server-side default values
the user never specified. Concretely close the awscc#206 reproduction
in which `awscc.s3.Bucket` plans show:

```
- transition_default_minimum_object_size: "all_storage_classes_128K"
```

even though no `.crn` source mentions the field.

The fix is *not* a per-resource allow list. It is a first-class
**provenance** concept on every attribute value: each value is either
**user-set** (the user wrote it in their `.crn`, or has at some past
apply written it) or **server-only** (the provider's read response
returned it but the user never wrote it). The differ ignores
`server-only` values; the rest of the system treats them transparently.

## Non-goals

- Migrating provider plugin protocols other than the WIT plugin
  contract.
- Solving the unrelated `~ {whole list element}` rendering opacity
  (carina#2881 owns that).
- Persisting provenance for nested fields *inside list-of-struct
  elements*. Identity matching for list elements is intentionally
  out of scope (see "Edge cases" below).

## Why provenance is a core concept, not a provider-side fix

Server-side defaults are a class-of-problem, not an awscc bug:

- AWS Cloud Control returns defaults for `S3.Bucket`,
  `EC2.SecurityGroup`, `IAM.Role.Path`, and many others.
- The native AWS SDK path (carina-provider-aws) has the same shape: any
  `describe_*` API may surface fields the user never set.
- Future providers (GCP, Azure, Kubernetes) will all hit the same
  asymmetry between desired-state (declarative, sparse) and read-state
  (full, server-augmented).

A truthful representation of "did the user write this?" therefore
belongs in the carina-core data model. Pushing it into each provider
forces re-implementation of the same logic at every provider boundary,
and silently strips the information when state crosses the WIT seam.

## Chosen approach: B2 — `Tracked<Value>` in core, WIT contract updated

Each entry in `State.attributes` carries a `Provenance` tag alongside
its `Value`. The tag is propagated end-to-end:

1. **WIT contract** introduces `tracked-attribute` and a `provenance`
   enum (`user-set` | `server-only`). The `state` record's
   `attributes` field becomes `list<tracked-attribute>`.

2. **carina-core** changes `State` to hold
   `attributes: HashMap<String, Tracked<Value>>` where
   `Tracked<T>(T, Provenance)`. Public APIs that previously returned
   `&Value` gain `Tracked` variants where callers need provenance
   visibility; existing call sites that don't care use a
   `.value()` accessor.

3. **State persistence (state v6)** adds a `"provenance"` field per
   attribute entry. Read of v5 maps every attribute to
   `Provenance::UserSet` for backward read compatibility — this is
   *not* a graceful migration of behavior (the project policy is "no
   backward compat"), it is a one-shot read step so existing state
   files don't catastrophically refuse to load. v6 is the only writable
   format; once a state file is touched, it is upgraded.

4. **Differ** filters `Provenance::ServerOnly` entries out of the diff:
   they neither produce `add`/`replace`/`remove` patch ops nor appear
   in the rendered plan. "Explicit unset" (saved had `UserSet`,
   desired no longer mentions it) still produces `remove` because
   provenance carries from saved-state.

5. **Provider boundary (read pipeline)** — each provider classifies
   fields returned from the cloud API:

   - field key in `desired` attrs → `UserSet`
   - else field key in `saved` attrs as `UserSet` → `UserSet`
     (carry forward; user owns it even if not in current desired)
   - else → `ServerOnly`

   Providers implement this independently in their own crate
   (CLAUDE.md / memory `feedback_provider_boundary_no_dedup` forbids
   sharing helpers across the aws/awscc boundary). carina-core
   provides the `Tracked` / `Provenance` types and a thin helper
   (`Tracked::classify`) but no provider-specific logic.

6. **Nested struct fields**: provenance is **per top-level attribute
   key only** in this design. The motivating field
   (`lifecycle_configuration.transition_default_minimum_object_size`)
   is inside a Struct, but provenance applies to the outer
   `lifecycle_configuration` key. To make the inner field invisible,
   the provider's read normalizer rebuilds the Struct value in two
   passes: classify the *whole* Struct's leaf fields by the same
   saved/desired comparison, then drop the leaves whose classification
   is `ServerOnly`. The Struct value finally placed in
   `Tracked<Value>` already has server-only leaves removed; the
   `Tracked` wrapper itself is `UserSet` because the user *did* write
   `lifecycle_configuration { ... }`.

   This split (top-level Tracked, leaf-level filter inside the
   provider) is deliberate: lifting Tracked to every leaf would
   cascade `Provenance` into `Value::Struct` / `Value::List` and
   bloat the WIT contract for marginal benefit.

## Key design decisions

### D1. Provenance lives on top-level attributes, not on every nested value

Alternative: put `Provenance` on every leaf inside `Value::Struct`,
`Value::Map`, `Value::List`. Rejected because:

- The `Value` enum is widely used (parser, differ, display, snapshot
  tests, plugin SDK). Re-typing every leaf forces a rewrite of all
  pattern matching.
- The motivating problem only requires removing leaves the user
  didn't write — the *structural shape* of the diff doesn't need
  per-leaf provenance once leaves are filtered at the provider
  boundary.
- A future requirement that genuinely needs leaf-level provenance
  (e.g. mixed user/server values inside a single Struct that the
  user partially specified) can be layered on top: extend `Value` to
  optionally carry provenance on Struct fields, without rewriting
  the top-level abstraction.

### D2. Explicit unset is detected via saved-state carry-forward

When `desired` no longer mentions a field that `saved` had as
`UserSet`, the differ must produce `- field`. Achieved naturally:
during plan computation, attributes present in `saved.attributes`
but absent from `desired.attributes` are added back with their
saved provenance. If `UserSet`, they appear as `Remove`. If
`ServerOnly`, they are dropped (server-side defaults disappearing
from a future read are not user-visible).

### D3. State v6 is a hard upgrade, not a migration

Memory rule "No backward compatibility — and don't mention it"
applies. v5→v6 read defaults all attributes to `UserSet`, but no
"migration path" docs are written; once carina writes the file it
is v6. Old carina binaries that read v6 will fail with
`UnsupportedVersion`, which is fine — the project is experimental.

### D4. Per-provider implementation, no shared helper

Each of carina-provider-aws and carina-provider-awscc implements
provenance classification independently. Memory rule
`feedback_provider_boundary_no_dedup` is in force: surface-level
similarity is incidental.

The classification logic is small (one HashMap lookup against
saved + desired) so duplication cost is low, and the providers
have different read pipelines (Cloud Control's flat JSON vs. the
AWS SDK's typed structs) where the classifier integrates.

### D5. Top-level provenance suffices for awscc#206 because the
        differ already iterates Struct leaves

The current awscc differ produces `- transition_default_minimum_object_size`
because it walks `lifecycle_configuration`'s Struct leaves and finds
a leaf in `actual` not in `desired`. If the provider boundary has
already removed that leaf from the actual `Value::Struct` (per the
two-pass strategy in approach step 6), the differ never sees it —
no `- field` line is emitted.

This means the `Tracked` wrapper at the top level is *necessary for
detecting explicit unset of whole attributes* (e.g. user removes
the entire `lifecycle_configuration` block), but the *spurious
nested leaf* problem is fixed by leaf filtering inside the
provider, no per-leaf Tracked needed.

## File / module touch map (anticipated)

### carina-plugin-wit (separate repo)

- `wit/types.wit`: introduce `tracked-attribute` record and
  `provenance` enum; change `state.attributes` type.
- Bump WIT contract version per repo conventions.

### carina (this repo for the wit-consumer side)

- `carina-core/src/value.rs` (or new `provenance.rs`): `Tracked<T>`
  and `Provenance` enums.
- `carina-core/src/provider.rs`: `State.attributes` type change.
  Update `BoxFuture` signatures of `Provider::read`, `update`, etc.
- `carina-core/src/differ/`: filter `ServerOnly` entries before
  diff, surface `UserSet`-only patches.
- `carina-state/`: state v6 schema. New `provenance` field per
  attribute. v5 reader path defaults to `UserSet`. CURRENT_VERSION
  bumped to 6.
- `carina-cli/src/display/`: snapshot tests need regeneration where
  output changes (the only intended change is *fewer* lines for
  server-default-bearing resources).
- `carina-plugin-host/`, `carina-plugin-sdk/`: bindgen regen
  for the new WIT shape; update the host-side conversions.
- `carina-provider-mock/`: minimal classifier (defaults all read
  attrs to `UserSet` since the mock has no real server defaults).

### carina-provider-awscc (separate repo)

- `carina-provider-awscc/src/provider/operations.rs`:
  - `map_aws_props_to_attributes` returns
    `(HashMap<String, Tracked<Value>>, ...)` after classification.
  - New `classify_struct_leaves` recurses into Struct values and
    drops leaves the user didn't write.
- `carina-provider-awscc/src/provider/normalizer.rs`:
  `restore_unreturned_attrs_impl` carries forward provenance from
  saved.
- Acceptance tests for the awscc#206 reproduction.

### carina-provider-aws (separate repo)

- Symmetric changes in the aws read pipeline. Implementation
  independent from awscc.

## Edge cases

- **Initial apply, no saved state**. `saved.attributes` is empty.
  Read result fields not in `desired` are classified `ServerOnly`.
  This is correct: on first apply the user has not "written" any
  field they didn't put in the `.crn`. Server defaults disappear
  from the diff from day one.

- **`tags` attribute**. Already special-cased in
  `map_aws_props_to_attributes` (skipped). Provenance for `tags`
  inherits from existing tag-merging logic; this design does not
  change tag handling.

- **List of Struct (e.g. `lifecycle_configuration.rules`)**. Out of
  scope. Each list element is treated atomically; if AWS adds a
  default leaf inside a user-written rule element, the leaf will
  still surface as a diff. This is documented and tracked
  separately if it becomes a problem.

- **Struct field with both user-set and server-set leaves**. The
  user-set leaves stay; the server-set leaves are removed before
  the differ sees them. The `Value::Struct` placed in `Tracked`
  contains only user-set leaves.

- **User explicitly sets a field to AWS's default value**. E.g. user
  writes `transition_default_minimum_object_size: "all_storage_classes_128K"`.
  Saved state will record it as `UserSet`. Reads continue to classify
  it as `UserSet` (because saved has it). It appears in the diff
  normally if changed. Correct behavior.

- **Migrating live `infra/` state from v5 to v6**. First plan after
  upgrade: every existing attribute is loaded as `UserSet` from v5,
  read-side classification then promotes server-only fields to
  `ServerOnly` and drops them. Net effect: the user sees the
  spurious `- field` lines disappear on the first plan after
  upgrading. No manual intervention required.

## Testing strategy

- **Unit tests** in `carina-core` for `Tracked`/`Provenance`
  serde round-trip, differ filtering, state v5→v6 read.
- **Provider unit tests** in awscc and aws for the classification
  logic against synthetic CloudControl / SDK responses.
- **Snapshot tests** in `carina-cli/tests/fixtures/plan_display/`
  with a new fixture covering the awscc#206 scenario: a `.crn`
  that does not set `transition_default_minimum_object_size` and a
  saved state that *also* doesn't set it, with a read response
  that does. Expected snapshot: no `- field` line.
- **Real-infra smoke** (manual, by user): run
  `aws-vault exec mizzy -- carina plan` against
  `carina-rs/infra/.../registry/dev/registry` and confirm
  `transition_default_minimum_object_size` no longer appears.

## Risks

- **WIT contract change blast radius**. Every provider plugin must
  rebuild against the new WIT. Existing pinned plugin binaries
  refuse to load. Acceptable per project policy.
- **State v5→v6 lock-in**. Once carina writes v6, older carina
  binaries can't read it. Standard for this project.
- **Differ behavior change is silent for users**. The intended
  effect is *fewer* spurious diff lines. Snapshot test coverage
  catches accidental over-filtering of legitimate diffs.
