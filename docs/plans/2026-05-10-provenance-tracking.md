# Implementation Plan — Attribute Provenance Tracking

<!-- derived-from ../specs/2026-05-10-provenance-tracking-design.md -->

## Goal

Implement the design in
`docs/specs/2026-05-10-provenance-tracking-design.md`. Final
acceptance: awscc#206 reproduction (`- transition_default_minimum_object_size`
in plan output) is gone, real-infra smoke confirms.

## Repo dependency order

```
carina-plugin-wit  ──►  carina  ──►  carina-provider-awscc
                                ──►  carina-provider-aws
```

Each repo step is one or more PRs. Carina downstream tasks block on
the WIT contract being merged. The two provider repos are mutually
independent and can run in parallel after carina lands.

## Task list

Each task corresponds to one GitHub Issue and one PR.

---

### Task 1/9 — wit: add `provenance` enum and `tracked-attribute` record

**Repo:** carina-plugin-wit
**Labels:** enhancement, task-1/9
**Blocked by:** none

**Goal:** Land the WIT contract changes. Pure schema PR; no consumer
updates yet (those follow in tasks 2+).

**Files:**
- Modify `wit/types.wit`

**Test:**
1. Author the WIT first; run `wit-bindgen rust --check wit/` (or
   whatever check the repo uses) to confirm it parses.
2. Add a doc-comment example to the new record in `types.wit`.

**Implementation:**
- Add `enum provenance { user-set, server-only }`.
- Add `record tracked-attribute { key: string, value: value, provenance: provenance }`.
- Change `state.attributes` from
  `list<tuple<string, value>>` to `list<tracked-attribute>`.
- Do NOT change `resource-def.attributes` — desired-state from the
  user is, by definition, all `user-set` and doesn't need the tag
  in the wire format. Providers attach provenance on the read side.

**Verification:**
```
git diff wit/types.wit
# WIT package check (whatever the repo's CI uses):
cargo check  # if a Rust harness exists
```
Add a CHANGELOG entry per repo conventions.

**Acceptance:**
- WIT file parses.
- All consumers (carina, carina-provider-aws, carina-provider-awscc)
  intentionally fail to build against this branch — that is the
  signal for tasks 2–9.

---

### Task 2/9 — carina-core: `Provenance` and `Tracked<T>` types

**Repo:** carina (this repo)
**Labels:** enhancement, rust, task-2/9
**Blocked by:** Task 1/9 merged

**Goal:** Introduce the core abstraction without yet rewiring `State`.

**Files:**
- Create `carina-core/src/provenance.rs`
- Modify `carina-core/src/lib.rs` (re-export)

**Test (write first):**
```rust
// carina-core/src/provenance.rs (in #[cfg(test)] mod)
#[test]
fn tracked_value_round_trips_user_set() {
    let t = Tracked::new(Value::String("x".into()), Provenance::UserSet);
    assert_eq!(t.value(), &Value::String("x".into()));
    assert_eq!(t.provenance(), Provenance::UserSet);
}

#[test]
fn provenance_serializes_kebab_case() {
    let json = serde_json::to_string(&Provenance::ServerOnly).unwrap();
    assert_eq!(json, "\"server-only\"");
}
```

**Implementation:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provenance {
    UserSet,
    ServerOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tracked<T> {
    value: T,
    provenance: Provenance,
}

impl<T> Tracked<T> {
    pub fn new(value: T, provenance: Provenance) -> Self { ... }
    pub fn user_set(value: T) -> Self { Self::new(value, Provenance::UserSet) }
    pub fn server_only(value: T) -> Self { Self::new(value, Provenance::ServerOnly) }
    pub fn value(&self) -> &T { &self.value }
    pub fn into_value(self) -> T { self.value }
    pub fn provenance(&self) -> Provenance { self.provenance }
    pub fn is_user_set(&self) -> bool { self.provenance == Provenance::UserSet }
}
```

**Verification:**
```
cargo nextest run -p carina-core provenance
cargo test --workspace --doc
cargo clippy -p carina-core --all-targets -- -D warnings
```

**Acceptance:**
- `cargo check -p carina-core` green.
- New unit tests pass.
- `Tracked` and `Provenance` exported from `carina_core::provenance`.

---

### Task 3/9 — carina-core: rewire `State.attributes` to `HashMap<String, Tracked<Value>>`

**Repo:** carina
**Labels:** enhancement, rust, task-3/9
**Blocked by:** Task 2/9 merged

**Goal:** Move the type change through `carina-core::provider::State`
and update every internal call site to compile.

**Files:**
- Modify `carina-core/src/provider.rs` (State struct definition)
- Modify every file in `carina-core/` that reads/writes
  `state.attributes` (parser, planner, plugin-host conversion, etc.).

  Use `cargo check -p carina-core` to enumerate; expected sites:
  - `carina-core/src/differ/comparison.rs`
  - `carina-core/src/differ/plan.rs`
  - `carina-core/src/normalize.rs` (if present)
  - any executor module that reads attrs

**Test (write first):**
```rust
// carina-core/tests/state_attributes_tracked.rs
#[test]
fn state_attribute_keeps_provenance_round_trip() {
    let mut s = State::new();
    s.attributes.insert(
        "name".into(),
        Tracked::user_set(Value::String("hi".into())),
    );
    let serialized = serde_json::to_string(&s).unwrap();
    let back: State = serde_json::from_str(&serialized).unwrap();
    assert!(back.attributes["name"].is_user_set());
}
```

**Implementation:**
- Change field type:
  ```rust
  pub struct State {
      pub identifier: Option<String>,
      pub attributes: HashMap<String, Tracked<Value>>,
      pub exists: bool,
  }
  ```
- For every `state.attributes.insert(k, v)` call where `v: Value`,
  replace with `state.attributes.insert(k, Tracked::user_set(v))`
  by default. Differ-specific code that already needs to distinguish
  goes in Task 4.

**Verification:**
```
cargo check -p carina-core
cargo nextest run -p carina-core
cargo clippy -p carina-core --all-targets -- -D warnings
```

**Acceptance:**
- `carina-core` compiles.
- All carina-core tests still pass (provenance default is UserSet,
  which is observationally identical to the pre-change behavior).
- Workspace likely *fails* to compile — downstream tasks fix it.

---

### Task 4/9 — carina-core differ: filter `ServerOnly` from diffs

**Repo:** carina
**Labels:** enhancement, rust, task-4/9
**Blocked by:** Task 3/9 merged

**Goal:** Make the differ skip `ServerOnly` entries when computing
patch ops and when rendering.

**Files:**
- Modify `carina-core/src/differ/comparison.rs`
- Modify `carina-core/src/differ/plan.rs` (or wherever `compute_patch`
  / `diff` lives — confirmed in Step 1 of design).

**Test (write first):**
```rust
// carina-core/src/differ/diff_tests.rs (additions)
#[test]
fn server_only_attr_in_actual_does_not_produce_remove_op() {
    let desired = state_with([("user_field", Tracked::user_set(...))]);
    let actual = state_with([
        ("user_field", Tracked::user_set(...)),
        ("server_default", Tracked::server_only(Value::String("x".into()))),
    ]);
    let patch = diff(&desired, &actual);
    assert!(patch.ops.iter().all(|op| op.key != "server_default"));
}

#[test]
fn user_set_attr_missing_from_desired_still_produces_remove() {
    let desired = state_with([]);
    let actual = state_with([
        ("user_field", Tracked::user_set(Value::String("x".into()))),
    ]);
    let patch = diff(&desired, &actual);
    assert!(patch.ops.iter().any(|op|
        op.key == "user_field" && op.kind == PatchOpKind::Remove
    ));
}
```

**Implementation:**
- In the patch-computation loop, when iterating `actual.attributes`,
  skip entries whose `Provenance::ServerOnly`.
- When iterating `desired.attributes`, all entries are by construction
  `UserSet`; just compare against actual's value field.
- Display layer reads only patch ops, so the existing display code
  needs no change — it only sees `UserSet` ops.

**Verification:**
```
cargo nextest run -p carina-core differ
cargo test --workspace --doc
cargo clippy -p carina-core --all-targets -- -D warnings
```

**Acceptance:**
- New tests pass.
- Existing differ tests still pass.

---

### Task 5/9 — carina-state: state v6 schema with provenance

**Repo:** carina
**Labels:** enhancement, rust, task-5/9
**Blocked by:** Task 3/9 merged (independent from Task 4)

**Goal:** Persist `Tracked<Value>` in the state file. Read v5 by
defaulting all attributes to `UserSet`.

**Files:**
- Modify `carina-state/src/state/mod.rs`
- Add `carina-state/tests/state_v6_round_trip.rs`
- Add `carina-state/tests/state_v5_to_v6_read.rs`

**Test (write first):**
```rust
// carina-state/tests/state_v6_round_trip.rs
#[test]
fn writing_then_reading_v6_preserves_provenance() {
    let mut state = StateFile::new();
    state.resources.insert(
        rid("aws", "s3.bucket", "x"),
        ResourceState {
            attributes: hashmap! {
                "name".into() => Tracked::user_set(Value::String("a".into())),
                "arn".into() => Tracked::server_only(Value::String("arn:...".into())),
            },
            exists: true,
            identifier: None,
        },
    );
    let json = serde_json::to_string(&state).unwrap();
    let back: StateFile = parse_state(&json).unwrap();
    let attrs = &back.resources[&rid("aws", "s3.bucket", "x")].attributes;
    assert_eq!(attrs["name"].provenance(), Provenance::UserSet);
    assert_eq!(attrs["arn"].provenance(), Provenance::ServerOnly);
}
```

```rust
// carina-state/tests/state_v5_to_v6_read.rs
#[test]
fn reading_v5_state_defaults_all_attributes_to_user_set() {
    let v5_json = r#"{
        "version": 5,
        "resources": {
            "aws.s3.bucket.x": {
                "attributes": {"name": {"String": "a"}},
                "exists": true,
                "identifier": null
            }
        }
    }"#;
    let state = parse_state(v5_json).unwrap();
    let attrs = &state.resources[&rid("aws", "s3.bucket", "x")].attributes;
    assert_eq!(attrs["name"].provenance(), Provenance::UserSet);
}
```

**Implementation:**
- Bump `CURRENT_VERSION = 6`.
- Adjust the serialized form of attributes to include the
  `"provenance"` field. Decide between:
  - inline tagged: `{"name": {"value": ..., "provenance": "user-set"}}`
  - flat with sentinel: `{"name": {"v": ..., "p": "user-set"}}`
  Pick inline tagged for readability; `Tracked<T>` derives Serde
  with field names `value` and `provenance`.
- In the v5 reader path, parse attributes as `HashMap<String, Value>`
  and wrap each in `Tracked::user_set` before constructing the v6
  in-memory state.

**Verification:**
```
cargo nextest run -p carina-state
cargo test --workspace --doc
cargo clippy -p carina-state --all-targets -- -D warnings
```

**Acceptance:**
- v6 round-trip preserves provenance.
- v5 read promotes everything to `UserSet`.
- `CURRENT_VERSION` is 6; writes always emit v6.

---

### Task 6/9 — carina-plugin-host / sdk: bindgen regen and conversions

**Repo:** carina
**Labels:** refactor, rust, task-6/9
**Blocked by:** Task 1/9 merged + Task 3/9 merged

**Goal:** Update WIT bindings consumers. The SDK side is what
plugins compile against; the host side is what carina-cli loads.

**Files:**
- Update `carina-plugin-wit` git submodule pointer (this repo
  embeds it; `git submodule update --remote`).
- Modify `carina-plugin-host/src/conversion.rs` (or wherever
  `state` ↔ `wit::State` conversions live).
- Modify `carina-plugin-sdk/src/conversion.rs` symmetrically.
- Update `carina-provider-mock/` to convert plain attrs to
  `Tracked::user_set` on read (mock has no real server defaults).

**Test (write first):**
```rust
// carina-plugin-host/tests/state_conversion.rs
#[test]
fn wit_state_with_server_only_attr_round_trips_to_core_state() {
    let wit_state = wit::State {
        identifier: None,
        exists: true,
        attributes: vec![
            wit::TrackedAttribute {
                key: "field".into(),
                value: wit::Value::String("x".into()),
                provenance: wit::Provenance::ServerOnly,
            },
        ],
    };
    let core_state = State::from_wit(wit_state);
    assert_eq!(
        core_state.attributes["field"].provenance(),
        Provenance::ServerOnly,
    );
}
```

**Implementation:**
- Map `wit::Provenance` ↔ `core::Provenance` 1:1.
- Map `wit::TrackedAttribute` ↔ `(String, Tracked<Value>)`.
- Update mock provider's `read_resource` to wrap returned attrs in
  `Tracked::user_set` (mock has no real server defaults to mark).

**Verification:**
```
cargo nextest run -p carina-plugin-host -p carina-plugin-sdk -p carina-provider-mock
cargo build -p carina-provider-mock --target wasm32-wasip2
cargo clippy --workspace --all-targets -- -D warnings
```

**Acceptance:**
- Workspace builds end-to-end.
- Mock plugin builds for `wasm32-wasip2`.

---

### Task 7/9 — carina-cli snapshot tests: refresh for unchanged-output cases

**Repo:** carina
**Labels:** test, rust, task-7/9
**Blocked by:** Tasks 4, 5, 6 merged

**Goal:** Confirm that no existing fixture's plan output changes
inadvertently. Add a new fixture for the awscc#206 scenario.

**Files:**
- Add `carina-cli/tests/fixtures/plan_display/server_only_struct_leaf/`:
  - `main.crn` — synthetic resource that does NOT set
    `transition_default_minimum_object_size`
  - `carina.state.json` — saved state without the field
  - The fixture uses a synthetic mock-provider resource shaped like
    awscc.s3.Bucket so the test stays in the carina repo without an
    awscc dependency.
- Modify `Makefile`: add `plan-server-only-leaf` target (CI
  invariant; see memory rule `feedback_makefile_for_fixtures`).
- Modify `carina-cli/src/plan_snapshot_tests.rs`: register fixture.

**Test:**
- Run `cargo nextest run -p carina-cli plan_snapshot` — fixture
  asserts that the rendered plan does NOT contain
  `- transition_default_minimum_object_size`.
- Run `cargo insta review` if any *other* snapshot drifts; the
  intended drift is zero (no other fixture exercises ServerOnly).

**Implementation:**
The fixture's `carina.state.json` should encode v6 directly with
`{"value": ..., "provenance": "user-set"}` per attribute, plus one
attribute with `"server-only"` to simulate a CloudControl read
result.

**Verification:**
```
cargo nextest run -p carina-cli plan_snapshot
make plan-fixtures
bash scripts/check-docs-drift.sh
bash scripts/check-snapshot-drift.sh  # if exists
```

**Acceptance:**
- New fixture's snapshot shows zero `-` lines for the server-only
  field.
- All other fixtures' snapshots unchanged.

---

### Task 8/9 — carina-provider-awscc: classifier in read pipeline

**Repo:** carina-provider-awscc
**Labels:** enhancement, rust, task-8/9
**Blocked by:** Tasks 1, 3, 6 merged (this repo updates its
`carina-core` and `carina-plugin-wit` git deps)

**Goal:** Classify CloudControl-returned attributes as
`UserSet` / `ServerOnly` at the awscc read boundary, including
recursively dropping `ServerOnly` leaves inside `Value::Struct`.

**Files:**
- Modify `carina-provider-awscc/src/provider/operations.rs`:
  - Change `map_aws_props_to_attributes` to take `desired` and
    `saved` attribute maps, and return
    `HashMap<String, Tracked<Value>>`.
  - Add `classify_struct_leaves(value: Value, ...) -> Value`
    that recurses into `Value::Struct` and drops fields not in
    `desired_struct_keys ∪ saved_struct_keys`.
- Modify `carina-provider-awscc/src/provider/mod.rs` /
  `read.rs` (whichever calls `map_aws_props_to_attributes`) to
  pass desired+saved.
- Modify `carina-provider-awscc/src/provider/normalizer.rs`:
  `restore_unreturned_attrs_impl` carries over saved
  `Tracked::provenance()` (don't blindly wrap as UserSet).

**Test (write first):**
```rust
// carina-provider-awscc/src/provider/operations.rs (#[cfg(test)])
#[test]
fn server_default_struct_leaf_dropped_when_user_did_not_set() {
    let attrs = lifecycle_configuration_schema();  // helper
    let props = json!({
        "LifecycleConfiguration": {
            "Rules": [...],
            "TransitionDefaultMinimumObjectSize": "all_storage_classes_128K"
        }
    });
    let desired = hashmap! {
        "lifecycle_configuration".into() => Value::Struct(...)
            // user wrote rules: [...] but NOT transition_default_*
    };
    let saved = HashMap::new();  // first apply
    let result = map_aws_props_to_attributes(
        &props, &attrs, "s3.Bucket", &desired, &saved,
    );
    let lc = result.get("lifecycle_configuration").unwrap();
    assert_eq!(lc.provenance(), Provenance::UserSet);
    let Value::Struct { fields, .. } = lc.value() else { panic!() };
    assert!(!fields.iter().any(|f| f.0 == "transition_default_minimum_object_size"));
}

#[test]
fn server_default_struct_leaf_kept_when_user_did_set() {
    // Same as above but `desired` has transition_default_*
    // → leaf survives in the Tracked<Value>.
}

#[test]
fn user_set_top_level_carry_forward_from_saved_when_desired_missing() {
    // Tests the explicit-unset detection: saved has key, desired
    // doesn't. Classifier must keep it as UserSet so the differ
    // can produce a Remove.
}
```

**Implementation:**
- Build a `desired_keys: HashSet<&str>` and `saved_keys:
  HashSet<&str>` from the inputs.
- Classify top-level: `UserSet` if key in either set, else
  `ServerOnly`.
- For Struct values, do the same comparison on each field name
  against the corresponding nested keys in desired/saved (extracted
  via the `Value::Struct` lookup); drop fields classified as
  `ServerOnly`.

**Verification:**
```
cargo nextest run -p carina-provider-awscc
cargo test --workspace --doc
cargo clippy -p carina-provider-awscc --all-targets -- -D warnings
cargo build -p carina-provider-awscc --target wasm32-wasip2 --release
bash scripts/check-docs-drift.sh
```

**Acceptance:**
- New unit tests pass.
- `wasm32-wasip2` build succeeds.
- The repo's CI gates (`scripts/check-*.sh`) all green.

---

### Task 9/9 — carina-provider-aws: classifier in read pipeline

**Repo:** carina-provider-aws
**Labels:** enhancement, rust, task-9/9
**Blocked by:** Tasks 1, 3, 6 merged (independent from Task 8)

**Goal:** Symmetric classifier on the SDK-direct read path. No
shared helper with awscc per CLAUDE.md / memory rule
`feedback_provider_boundary_no_dedup`.

**Files:**
- Identify aws read pipeline (`carina-provider-aws/src/provider/`,
  the SDK adapter functions per resource).
- Add `classify_attrs(read_attrs, desired, saved) ->
  HashMap<String, Tracked<Value>>` per resource adapter or as a
  small private helper in this crate.

**Test (write first):**
For one representative resource (e.g. `aws.s3.Bucket` or
`aws.iam.Role`), add a unit test mirroring task 8's structure:
read returns a server-default field, classifier marks it
`ServerOnly`.

**Implementation:**
- Same classification rule as awscc (`UserSet` if key in desired or
  saved; else `ServerOnly`). Implementation is independent of awscc
  by design.
- Apply Struct-leaf filtering only where the aws SDK's typed
  structs flatten to `Value::Struct` in the same shape.

**Verification:**
```
cargo nextest run -p carina-provider-aws
cargo test --workspace --doc
cargo clippy -p carina-provider-aws --all-targets -- -D warnings
cargo build -p carina-provider-aws --target wasm32-wasip2 --release
```

**Acceptance:**
- aws provider compiles against the new core.
- The representative resource's unit test passes.
- wasm build succeeds.

---

## Final acceptance (manual, by user)

After all 9 tasks are merged and the carina CLI is rebuilt:

```
cd /Users/mizzy/src/github.com/carina-rs/infra
aws-vault exec mizzy -- carina plan registry/dev/registry
```

Expected: no `- transition_default_minimum_object_size` line. The
state file at `registry/dev/registry/carina.state.json` will be
upgraded to v6 on the first apply after the change.

awscc#206 closes when this acceptance is reported.

## Cross-cutting notes

- **Multi-repo PR coordination**. Tasks 1, 8, 9 happen in repos
  outside this one. The PR descriptions for 2–7 reference the WIT
  PR (Task 1) and the carina-plugin-wit submodule SHA. Task 6
  bumps the submodule.
- **No backward compat docs**. State v6 is a hard upgrade. WIT
  contract bump means old plugins fail to load. Memory rule
  `feedback_no_backward_compat` applies — do not write migration
  guides.
- **Real-infra smoke at Task 8 only**. Acceptance for the awscc#206
  reproduction belongs to the awscc PR (Task 8). Carina-side PRs
  (Tasks 2–7) are verified by unit + snapshot tests; they don't
  need a real-AWS run.
