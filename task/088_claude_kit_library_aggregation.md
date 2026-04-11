# Create `claude_kit` library super-crate — aggregate all non-CLI crates

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)
- **Depends on:** TSK-087 — `claude_assets_core` must exist before `claude_kit` can aggregate it

## Goal

Create a new library crate `claude_kit` in `module/` that aggregates and re-exports every non-CLI crate of the workspace (`claude_common` + five `*_core` crates, including `claude_assets_core` from TSK-087) as short-named submodules — so downstream library consumers can depend on one aggregator crate instead of six individual ones, mirroring how `claude_tools` (clt) aggregates CLI crates at the binary layer (Motivated: reduce dependency fan-out and provide one canonical library entry point; Observable: `cargo add claude_kit` pulls in all non-CLI functionality, `use claude_kit::storage::...` and `use claude_kit::assets::...` compile; Scoped: pure library aggregation crate, no new logic, no binary target; Testable: `cargo check -p claude_kit && cargo nextest run -p claude_kit`).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_kit/Cargo.toml` — new library crate with path deps on `claude_common`, `claude_storage_core`, `claude_profile_core`, `claude_runner_core`, `claude_manager_core`, `claude_assets_core`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_kit/src/lib.rs` — re-export each aggregated crate under a short module alias (e.g., `pub use ::claude_storage_core as storage;`, `pub use ::claude_assets_core as assets;`)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_kit/readme.md` — purpose, aggregation map, usage example, distinguishing note vs. `claude_tools`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_kit/tests/smoke.rs` — compile-time smoke test referencing one public symbol from each of the 6 aggregated crates
- `/home/user1/pro/lib/wip_core/claude_tools/dev/Cargo.toml` — add `"module/claude_kit"` to `workspace.members` (alphabetical position)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/readme.md` — register `claude_kit/` in the parent responsibility table distinct from `claude_tools/`

## Out of Scope

- New functionality — `claude_kit` is a pure re-export crate; no new types, traits, or functions
- CLI crates (`claude_storage`, `claude_profile`, `claude_runner`, `claude_manager`) — kit is library-only
- `claude_tools` super-app — binary aggregator is unchanged
- Any modification to the source of aggregated crates
- The multi-artifact installer crate (separate task)
- Documentation under `docs/` beyond crate-local `readme.md`

## Description

The workspace has a CLI super-app (`claude_tools`, binary `clt`) that aggregates every CLI crate into a single user-facing entry point. There is **no equivalent for library consumers**. A downstream crate that wants to use `claude_storage_core` + `claude_profile_core` + `claude_runner_core` must individually declare three path/workspace dependencies, leading to brittle version alignment and dependency fan-out.

`claude_kit` fills that gap. It is a **pure re-export crate** that depends on every non-CLI crate (`claude_common` Layer 0 + five `*_core` Layer 1 crates, including `claude_assets_core` from TSK-087) and exposes them under short aliases:

```rust
pub use ::claude_common         as common;
pub use ::claude_storage_core   as storage;
pub use ::claude_profile_core   as profile;
pub use ::claude_runner_core    as runner;
pub use ::claude_manager_core   as manager;
pub use ::claude_assets_core    as assets;
```

Downstream consumers write `use claude_kit::storage::SessionManager;` or `use claude_kit::assets::ArtifactKind;` and pull the entire non-CLI layer transitively.

The workspace then has a **two-track aggregation pattern**:
- `claude_tools` — aggregates CLI binaries (user-facing `clt`)
- `claude_kit` — aggregates non-CLI libraries (developer-facing `use claude_kit::*`)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- `claude_kit` is `lib`-only — no `[[bin]]` target, no `main.rs`, no CLI dependency (`unilang`, `clap`, etc.)
- Re-exports use `pub use ... as <alias>;`; zero type/trait/function redefinition in `claude_kit`
- Module aliases must be short (`storage` not `claude_storage_core`) for ergonomic consumer imports
- Version pinned to workspace via `version.workspace = true`
- `readme.md` must include a responsibility table row distinguishing `claude_kit` (library aggregator) from `claude_tools` (binary aggregator)
- No `cargo fmt` — follow custom codestyle from applicable rulebooks
- Error handling follows workspace pattern via `error_tools` if any wrapper code is ever added (not expected in v1)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` on crate organization, `files_structure.rulebook.md` on readme responsibility tables, `organizational_principles.rulebook.md` on Anti-Duplication and Unique Responsibility.
2. **Read existing aggregation pattern** — Read `module/claude_tools/Cargo.toml` and `module/claude_tools/src/lib.rs` (or `src/main.rs`) to understand the CLI aggregation pattern; replicate the structure for libraries.
3. **Read each aggregated crate's public surface** — Read `lib.rs` of `claude_common`, `claude_storage_core`, `claude_profile_core`, `claude_runner_core`, `claude_manager_core`, `claude_assets_core` to confirm at least one known public symbol per crate for the smoke test. (`claude_assets_core` must exist first — TSK-087 prerequisite.)
4. **Write failing smoke test** — Create `module/claude_kit/tests/smoke.rs` referencing one well-known symbol from each of the 6 aggregated crates (e.g., `claude_kit::common::ClaudePaths`, `claude_kit::assets::ArtifactKind`). Test must fail to compile until re-exports exist.
5. **Create crate skeleton** — `module/claude_kit/Cargo.toml` with `[package]` (workspace version/edition), `[lib]` (no `[[bin]]`), and `[dependencies]` listing the 6 path deps.
6. **Create `src/lib.rs`** — crate-level doc comment explaining aggregation purpose and the two-track pattern; six `pub use` statements mapping each crate to a short alias.
7. **Create `readme.md`** — responsibility table row (single-sentence responsibility), usage example, and explicit "see also: claude_tools for CLI aggregator" cross-reference.
8. **Register in workspace root** — add `"module/claude_kit"` to `Cargo.toml` `workspace.members` in alphabetical position.
9. **Register in parent `module/readme.md`** — add a row for `claude_kit/` to the responsibility table.
10. **Validate** — Run `w3 .test level::3` (or `ctest3`). All tests including new smoke test must pass; clippy clean; doc-tests clean; zero warnings.
11. **Walk Validation Checklist** — check every item. Every answer must be YES.
12. **Update task status** — mark complete in `task/readme.md` index.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `cargo check -p claude_kit` | Fresh crate, all 5 deps resolved | Compiles with zero warnings |
| `use claude_kit::common::ClaudePaths;` in smoke test | common re-export active | Resolves, compiles |
| `use claude_kit::storage::<public type>;` | storage re-export active | Resolves, compiles |
| `use claude_kit::profile::<public type>;` | profile re-export active | Resolves, compiles |
| `use claude_kit::runner::<public type>;` | runner re-export active | Resolves, compiles |
| `use claude_kit::manager::<public type>;` | manager re-export active | Resolves, compiles |
| `cargo doc -p claude_kit` | lib.rs doc comments present | Docs generated; module index lists all 6 aggregated crates |
| `grep -c '\[\[bin\]\]' module/claude_kit/Cargo.toml` | claude_kit is library-only | Zero binary targets |
| `grep -cE '^pub (fn\|struct\|trait\|enum\|type) ' module/claude_kit/src/lib.rs` | Pure re-export crate | 0 locally-defined public items |
| `w3 .test level::3` | Full workspace | All tests pass; no regressions in other crates |

## Acceptance Criteria

- `module/claude_kit/` exists as a pure library crate (no binary target, no CLI framework dependency)
- `claude_kit` declares exactly 6 path dependencies: `claude_common`, `claude_storage_core`, `claude_profile_core`, `claude_runner_core`, `claude_manager_core`, `claude_assets_core`
- `claude_kit::common`, `claude_kit::storage`, `claude_kit::profile`, `claude_kit::runner`, `claude_kit::manager`, `claude_kit::assets` resolve in downstream code via `pub use ... as <alias>`
- `RUSTFLAGS="-D warnings" cargo check -p claude_kit` succeeds with zero warnings
- `cargo doc -p claude_kit` generates documentation with a module index listing all 6 aggregated crates
- `module/claude_kit/readme.md` has a responsibility table entry and a "see also: claude_tools" cross-reference
- `module/readme.md` has a row for `claude_kit/` distinct from `claude_tools/`
- Smoke test in `tests/smoke.rs` references one public symbol from each of the 6 aggregated crates and passes
- `w3 .test level::3` passes for the whole workspace with no regressions in previously-passing crates
- Zero new types, traits, or functions defined in `claude_kit` — re-exports only
- Workspace root `Cargo.toml` includes `"module/claude_kit"` in `workspace.members` in alphabetical order

## Validation

### Checklist

Desired answer for every question is YES.

**Crate structure**
- [ ] Does `module/claude_kit/Cargo.toml` exist with `[lib]` only (no `[[bin]]`)?
- [ ] Does the crate declare exactly the 6 expected path dependencies (including `claude_assets_core`)?
- [ ] Is `version.workspace = true` used for version alignment?

**Re-exports**
- [ ] Does `src/lib.rs` re-export each aggregated crate under a short module alias?
- [ ] Does each re-export use the `pub use ::<crate> as <alias>;` pattern?
- [ ] Are zero new types/traits/functions defined in `claude_kit`?

**Documentation**
- [ ] Does `src/lib.rs` have a crate-level doc comment explaining aggregation purpose?
- [ ] Does `readme.md` include a responsibility table row and a usage example?
- [ ] Is `claude_kit` distinguished from `claude_tools` in parent `module/readme.md`?

**Testing**
- [ ] Does `tests/smoke.rs` reference a public symbol from each of the 6 aggregated crates (including `claude_kit::assets`)?
- [ ] Does `w3 .test level::3` pass for the whole workspace?

**Workspace registration**
- [ ] Is `"module/claude_kit"` listed in root `Cargo.toml` `workspace.members`?
- [ ] Is the members list still alphabetical?

**Out of Scope confirmation**
- [ ] Are all 4 CLI crates (`claude_storage`, `claude_profile`, `claude_runner`, `claude_manager`) unchanged?
- [ ] Is `claude_tools` super-app unchanged?
- [ ] Are no new symbols defined in `claude_kit` beyond re-exports?

### Measurements

**M1 — Crate compiles cleanly**
Command: `RUSTFLAGS="-D warnings" cargo check -p claude_kit 2>&1 | tail -5`
Before: crate does not exist. Expected: `Finished`. Deviation: any warning or error.

**M2 — Smoke test passes**
Command: `cargo nextest run -p claude_kit --test smoke 2>&1 | tail -5`
Before: test does not exist. Expected: `test result: ok. N passed`. Deviation: compile or runtime failure.

**M3 — Library only, zero binary targets**
Command: `grep -c '\[\[bin\]\]' module/claude_kit/Cargo.toml`
Before: file does not exist. Expected: 0. Deviation: ≥1 (crate accidentally became a binary).

**M4 — Dependency count correct**
Command: `grep -cE 'claude_common|claude_storage_core|claude_profile_core|claude_runner_core|claude_manager_core|claude_assets_core' module/claude_kit/Cargo.toml`
Before: file does not exist. Expected: ≥6. Deviation: <6 (missing aggregation — ensure TSK-087 completed first).

**M5 — Full workspace verification**
Command: `w3 .test level::3 2>&1 | tail -5`
Before: baseline passing. Expected: all tests pass, no regressions. Deviation: any failure in previously-passing crate.

### Anti-faking checks

**AF1 — No re-implementation**
Check: `grep -cE '^pub (fn|struct|trait|enum|type) ' module/claude_kit/src/lib.rs`
Expected: 0. Why: `claude_kit` must be a pure re-export crate; any locally-defined public item indicates scope creep beyond aggregation and violates the Unique Responsibility Principle (aggregation only).

**AF2 — Aggregation completeness**
Check: `grep -c 'pub use ::claude_' module/claude_kit/src/lib.rs`
Expected: ≥6. Why: confirms every non-CLI crate is re-exported (including `claude_assets_core` from TSK-087); otherwise the aggregator is incomplete and consumers still need multiple deps.

**AF3 — No CLI crate dependency**
Check: `grep -cE '"claude_storage"|"claude_profile"|"claude_runner"|"claude_manager"|"claude_tools"' module/claude_kit/Cargo.toml`
Expected: 0. Why: `claude_kit` is library-only; depending on CLI crates would duplicate `claude_tools`' role and violate Unique Responsibility (two aggregators for the same domain).

**AF4 — Workspace registration**
Check: `grep -c '"module/claude_kit"' Cargo.toml`
Expected: 1. Why: confirms actual workspace registration, not just file creation in isolation (orphan crate).

**AF5 — Smoke test hits every alias**
Check: `grep -cE 'claude_kit::(common|storage|profile|runner|manager|assets)' module/claude_kit/tests/smoke.rs`
Expected: ≥6. Why: smoke test must touch each alias, not just one; otherwise a broken re-export (including the new `assets` alias) could silently ship.

## Outcomes

[Empty — populated upon task completion]
