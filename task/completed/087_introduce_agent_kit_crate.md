# TSK-087: Introduce `agent_kit` aggregation crate

## Goal

Add a new `agent_kit` library crate at Layer 2 that re-exports all core crates
(`claude_common`, `claude_storage_core`, `claude_profile_core`, `claude_manager_core`,
`claude_runner_core`) under feature-gated namespaced modules, giving downstream
consumers a single versioned dependency with selective domain inclusion via cargo
features — zero own logic, pure facade.

## Motivation

Consumers of the workspace currently must list 5 separate core crate dependencies
and coordinate their versions independently. An aggregation crate named `agent_kit`
matches the workspace identity, provides a single entry point, and lets callers pay
only for the domains they need via explicit feature flags.

## Architecture

### Layer position

Layer 2 — pure re-export facade. Depends only on Layer 0 + Layer 1. No dependency
on any Layer 2 CLI crate (`claude_profile`, `claude_storage`, `claude_runner`,
`claude_manager`) or Layer 3 (`claude_tools`).

### Feature graph

```toml
[features]
default = []
full    = [ "common", "storage", "profile", "runner", "manager" ]
enabled = [ "full" ]

common  = [ "dep:claude_common" ]
storage = [ "dep:claude_storage_core" ]
profile = [ "dep:claude_profile_core" ]
runner  = [ "dep:claude_runner_core" ]
manager = [ "dep:claude_manager_core" ]
```

`common` is a first-class feature because `claude_common` is independently useful
(`ClaudePaths`, process utilities) without pulling in any domain logic.
`profile`/`runner`/`manager` pull in `claude_common` transitively (their own
`[dependencies]` section is non-optional), so they do NOT need to list `common`
in their feature deps.

### Re-export module structure

```
agent_kit::common   ← claude_common      (feature = "common")
agent_kit::storage  ← claude_storage_core (feature = "storage")
agent_kit::profile  ← claude_profile_core (feature = "profile")
agent_kit::runner   ← claude_runner_core  (feature = "runner")
agent_kit::manager  ← claude_manager_core (feature = "manager")
```

### Core invariant

Zero own logic. No structs, traits, functions, enums, or type aliases defined in
`agent_kit` source. Every public item is a direct re-export from a core crate.

## In Scope

**New files (create all in same session):**

| Path | Responsibility |
|------|----------------|
| `module/agent_kit/Cargo.toml` | Crate manifest with feature-gated optional deps |
| `module/agent_kit/src/lib.rs` | Feature-gated `pub mod` re-exports + full crate doc |
| `module/agent_kit/readme.md` | User-facing docs: Responsibility Table, features, usage |
| `module/agent_kit/docs/feature/001_aggregation.md` | FR spec: facade behavior |
| `module/agent_kit/docs/invariant/001_no_own_logic.md` | Invariant: zero own types/fns |
| `module/agent_kit/tests/integration/facade_test.rs` | Compilation smoke tests per feature |
| `module/agent_kit/tests/readme.md` | Test suite index |

**Modified files:**

| Path | Change |
|------|--------|
| `Cargo.toml` | Add `"module/agent_kit"` to `[workspace] members`; add `[workspace.dependencies.agent_kit]` |
| `module/readme.md` | Add `agent_kit` row |
| `readme.md` | Add `agent_kit` row to Crates table; update Architecture diagram |
| `locales.md` | Add locale entry (Layer 2, lib, last_active = today) |
| `task/readme.md` | Register TSK-087 |

## Out of Scope

- Re-exporting Layer 2 CLI crates (`claude_profile`, `claude_storage`, `claude_runner`,
  `claude_manager`) — they carry unilang/CLI machinery unsuitable for library consumers
- Re-exporting `claude_tools` (Layer 3 super-app binary)
- Any own implementations, type aliases, newtypes, or wrapper types in `src/`
- `build.rs`, unilang YAML, or CLI binary (`[[bin]]`)
- Changing any existing crate

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Feature-gated re-exports: each `pub mod` is `#[cfg(feature = "…")]`
- `src/lib.rs` top-level doc comment includes a feature table
- `--no-default-features` compile must succeed with zero runtime deps
- L3 verification (`ctest3`) passes on the full workspace

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` for
   `src/lib.rs` formatting (2-space indent, `mod private {}` inline blocks),
   `crate_distribution.rulebook.md` for feature gating conventions, and
   `files_structure.rulebook.md` § Directory Responsibility Table for readme
   structure.

2. **Create crate skeleton** — Create `module/agent_kit/` with `readme.md` (add
   `agent_kit` row to `module/readme.md` first per File & Directory Creation
   Protocol). Skeleton: `Cargo.toml`, `src/lib.rs` (empty, just `//! TODO`).

3. **Write `Cargo.toml`** — Use workspace inheritance (`version.workspace = true`,
   etc.); set `description`, `readme`; all 5 core crates as `optional = true` under
   `[dependencies]`; feature graph exactly as shown in Architecture above; no
   `[[bin]]`, no `build.rs`, no `build-dependencies`.

4. **Register in workspace `Cargo.toml`** — Add `"module/agent_kit"` to
   `[workspace] members` list (alphabetical by first word: between `claude_tools`
   and any future entries). Add `[workspace.dependencies.agent_kit]` entry with
   `path`, `version = "~1.0.0"`, `default-features = false`.

5. **Write `src/lib.rs`** — Top-level `//!` doc with:
   - one-line crate description
   - Feature Flags table (Feature | Activates | Description)
   - Usage example in a `no_run` code block
   Then one `pub mod` block per feature, each containing only `pub use crate::*`
   from the corresponding crate. Follow 2-space indent, `mod private {}` at bottom
   for any needed internal items (likely empty).

6. **Write `readme.md`** — Responsibility Table (files/dirs), feature table with
   cargo snippet, usage example, architecture note (layer, zero-own-logic).

7. **Create `docs/` structure** — Two files:
   - `docs/feature/001_aggregation.md` — FR-1 through FR-5 (see Requirements below)
   - `docs/invariant/001_no_own_logic.md` — INV-1 through INV-3 (see below)

8. **Update workspace `readme.md`** — Add `agent_kit` row to Crates table (after
   `claude_manager`, before `claude_tools`; cmd = `—`; layer = 2; responsibility =
   "Library facade re-exporting all Layer 0–1 core crates"). Update Architecture
   text diagram to show `agent_kit (library)` alongside the Layer 2 CLI crates.

9. **Update `locales.md`** — Add row 11: `module/agent_kit`, `agent_kit`,
   `rust_crate`, `rs`, "Library facade re-exporting all Layer 0–1 core crates",
   `N`, today's date.

10. **RED phase** — Write `tests/integration/facade_test.rs` with 5 compile-check
    tests (one per feature module). Confirm they fail before implementation is
    wired (cargo nextest run -p agent_kit should error on missing items).

11. **GREEN phase** — Wire up re-exports, run `ctest3` on full workspace. All tests
    must pass. Zero warnings.

12. **Walk Validation Checklist** — every item must be YES. Fix any NO before
    marking done.

13. **Mark TSK-087 ✅** — update `task/readme.md` Status Distribution and task row.

## Feature Spec (for `docs/feature/001_aggregation.md`)

| ID | Requirement |
|----|-------------|
| FR-1 | When feature `common` is enabled, `agent_kit::common` re-exports all public items from `claude_common` |
| FR-2 | When feature `storage` is enabled, `agent_kit::storage` re-exports all public items from `claude_storage_core` |
| FR-3 | When feature `profile` is enabled, `agent_kit::profile` re-exports all public items from `claude_profile_core` |
| FR-4 | When feature `runner` is enabled, `agent_kit::runner` re-exports all public items from `claude_runner_core` |
| FR-5 | When feature `manager` is enabled, `agent_kit::manager` re-exports all public items from `claude_manager_core` |
| FR-6 | With no features enabled, the crate compiles with zero runtime dependencies |
| FR-7 | Feature `full` enables all five domain modules simultaneously |
| FR-8 | Enabling `storage` does NOT activate `claude_common` as a runtime dependency |
| FR-9 | Each feature is independently activatable without enabling unrelated features |

## Invariant Spec (for `docs/invariant/001_no_own_logic.md`)

| ID | Invariant |
|----|-----------|
| INV-1 | `src/` contains no `pub struct`, `pub fn`, `pub trait`, `pub enum`, or `pub type` definitions |
| INV-2 | All public items exported by `agent_kit` originate from a core crate |
| INV-3 | `agent_kit` has no dependency on any Layer 2 or Layer 3 crate |

## Acceptance Criteria

- `cargo check -p agent_kit --no-default-features` exits 0
- `cargo check -p agent_kit --features full` exits 0
- All five `agent_kit::{domain}::*` modules accessible with correct features
- `grep -rn "^pub struct\|^pub fn\|^pub trait\|^pub enum\|^pub type" module/agent_kit/src/` returns empty
- `docs/feature/001_aggregation.md` documents FR-1 through FR-9
- `docs/invariant/001_no_own_logic.md` documents INV-1 through INV-3
- Workspace `readme.md` Crates table includes `agent_kit`
- `locales.md` includes `agent_kit`
- `ctest3` passes on the full workspace

## Validation Checklist

**Compilation**
- [ ] `cargo check -p agent_kit --no-default-features` passes
- [ ] `cargo check -p agent_kit --features full` passes
- [ ] `cargo check -p agent_kit --features common` passes
- [ ] `cargo check -p agent_kit --features storage` passes
- [ ] `cargo check -p agent_kit --features profile` passes
- [ ] `cargo check -p agent_kit --features runner` passes
- [ ] `cargo check -p agent_kit --features manager` passes

**Feature isolation**
- [ ] `cargo tree -p agent_kit --features storage` does NOT list `claude_common`
- [ ] `cargo tree -p agent_kit --features profile` DOES list `claude_common` (transitive)

**Zero own logic**
- [ ] `grep -rn "^pub struct\|^pub fn\|^pub trait\|^pub enum\|^pub type" module/agent_kit/src/` is empty

**Documentation**
- [ ] `module/agent_kit/readme.md` has Responsibility Table
- [ ] `module/agent_kit/readme.md` has Feature Flags table with cargo snippet
- [ ] `docs/feature/001_aggregation.md` has FR-1 through FR-9
- [ ] `docs/invariant/001_no_own_logic.md` has INV-1 through INV-3
- [ ] Workspace `readme.md` Crates table row exists for `agent_kit`
- [ ] `locales.md` row exists for `agent_kit`
- [ ] `module/readme.md` row exists for `agent_kit`

**Full suite**
- [ ] `ctest3` passes on entire workspace

## Validation Procedure

**M1 — Zero-dep compile**
```bash
cargo check -p agent_kit --no-default-features
```
Expected: exit 0. Deviation: dependency leaked into default feature set.

**M2 — Full-features compile**
```bash
cargo check -p agent_kit --features full
```
Expected: exit 0. Deviation: feature graph or re-export path misconfigured.

**M3 — No own logic**
```bash
grep -rn "^pub struct\|^pub fn\|^pub trait\|^pub enum\|^pub type" module/agent_kit/src/
```
Expected: empty output. Deviation: own types crept into facade crate.

**M4 — Feature isolation: storage excludes claude_common**
```bash
cargo tree -p agent_kit --features storage --no-default-features 2>&1 | grep claude_common
```
Expected: no output. Deviation: common pulled in when not requested.

**M5 — Feature isolation: profile includes claude_common transitively**
```bash
cargo tree -p agent_kit --features profile --no-default-features 2>&1 | grep claude_common
```
Expected: match found. Deviation: transitive dep missing — profile_core Cargo.toml changed.

**AF1 — Re-exports actually resolve**
Write in a test or doc-test:
```rust,no_run
use agent_kit::profile::TokenStatus;
use agent_kit::runner::ClaudeCommand;
```
With `features = ["profile", "runner"]`. Expected: compiles. Why: file existence
alone doesn't prove re-export paths are correct.

**AF2 — Workspace member registered**
```bash
grep "agent_kit" Cargo.toml
```
Expected: `"module/agent_kit"` in members list AND `[workspace.dependencies.agent_kit]`
section present.

## Completion

**Completed:** 2026-04-11

All acceptance criteria met. All 9 FR requirements documented. All 3 INV invariants documented.
`ctest3` passes workspace-wide (1549/1550 — one pre-existing `b13` failure in `claude_storage`,
confirmed pre-existing via `git stash` verification, out of scope). Feature isolation verified:
`storage` excludes `claude_common`; `profile` includes it transitively. Zero own logic confirmed.
