# Create agent_inventory crate — agent-agnostic asset discovery with Claude Code adapter

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Create a Layer 1 `agent_inventory` crate providing an `AgentAdapter` trait and Claude Code adapter so that consumer workspaces can discover assets across multiple AI coding agents in a unified flat table (Motivated: kbase needs cross-agent asset listing; Observable: new crate compiles, adapter returns correct entries, tests pass; Scoped: crate creation + workspace integration + dream re-export only; Testable: `cargo nextest run -p agent_inventory --all-features`).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/` — new crate directory
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/Cargo.toml` — crate manifest with `claude_code` feature
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/lib.rs` — public API surface
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/adapter.rs` — `AgentAdapter` trait
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/entry.rs` — `AssetEntry`, `AssetKind`, `SyncStatus`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/inventory.rs` — `Inventory` registry
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/claude_code.rs` — Claude Code adapter
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/src/error.rs` — `InventoryError` type
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/readme.md` — crate readme
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/agent_inventory/tests/` — integration tests
- `/home/user1/pro/lib/wip_core/claude_tools/dev/Cargo.toml` — add workspace member + dependency entry
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/Cargo.toml` — add `inventory` feature
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/dream/src/lib.rs` — add `inventory` re-export module
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/readme.md` — add agent_inventory row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/readme.md` — update Crates table + Architecture diagram

## Out of Scope

- Documentation updates (already completed by doc_tsk — see `docs/feature/002_agent_inventory.md`)
- kbase `.skills` command integration (covered in T-395)
- Codex or Cursor adapters (future work, architecture supports it)
- CLI binary for agent_inventory (Layer 1 — no CLI)

## Description

The workspace currently has 6 Layer 1 domain crates, all focused on Claude Code. Adding `agent_inventory` introduces an agent-agnostic abstraction layer: an `AgentAdapter` trait that any AI coding agent can implement to expose its assets (skills, rules, commands, etc.) in a normalized `AssetEntry` format.

The first (and currently only) adapter wraps `claude_assets_core::registry` behind a `claude_code` feature gate. It converts `ArtifactKind` → `AssetKind` and computes `SyncStatus` from the combination of `InstallStatus` and source existence: `Synced` (symlink + source), `NotInstalled` (source only), `Orphaned` (symlink but source deleted). The `Inventory` struct collects adapters and merges their results into a single `Vec<AssetEntry>`, enabling flat-table output across all registered agents.

The crate follows the established workspace pattern: Layer 1 (zero CLI framework deps), workspace lints inherited, `#[inline]` on public items, `missing_docs` warning. It gets re-exported via `dream` under an `inventory` feature flag.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Layer 1 constraint: no `unilang`, no `clap`, no CLI framework dependencies
- Privacy invariant: crate has zero knowledge of consumer workspaces (invariant/001)
- Crate layering: `agent_inventory` depends on `claude_assets_core` (same layer — permitted), not on any Layer 2 crate (pattern/001)
- Workspace lints: inherit `[workspace.lints]` — `missing_docs`, `pedantic`, `missing_inline_in_public_items`
- Custom codestyle: 2-space indentation, spaces inside delimiters, no `cargo fmt`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code_design.rulebook.md constraints on trait design and error handling.
2. **Read documentation** — Read `docs/feature/002_agent_inventory.md` as source of truth for expected types and adapter trait.
3. **Read existing patterns** — Read `module/claude_assets_core/src/registry.rs` (list_all API), `module/claude_assets_core/src/artifact.rs` (ArtifactKind enum), `module/claude_assets_core/src/paths.rs` (AssetPaths::from_env), `module/claude_assets_core/src/error.rs` (AssetError). Read `module/dream/src/lib.rs` and `module/dream/Cargo.toml` for re-export pattern.
4. **Create crate scaffold** — Create `module/agent_inventory/Cargo.toml` with workspace version/edition/lints, `claude_code` optional feature gating `claude_assets_core` dependency. Create `module/agent_inventory/readme.md`.
5. **Write failing tests** — Create `module/agent_inventory/tests/` with tests covering: `AssetKind` variants, `AssetEntry` construction, `Inventory::new()` returns empty, Claude Code adapter returns entries when `$PRO_CLAUDE` is set (use tempdir), adapter name is `"claude_code"`.
6. **Implement core types** — Create `src/entry.rs` (`AssetEntry` with agent/kind/name/status fields, `AssetKind` enum, `SyncStatus` enum with `Synced`/`NotInstalled`/`Orphaned` variants and emoji `Display` impl: ✅/⬇️/⚠️), `src/error.rs` (`InventoryError`), `src/adapter.rs` (`AgentAdapter` trait).
7. **Implement inventory registry** — Create `src/inventory.rs` (`Inventory` with `register()` and `list_all()` that merges adapter results).
8. **Implement Claude Code adapter** — Create `src/claude_code.rs` gated under `#[cfg(feature = "claude_code")]`. Wrap `claude_assets_core::registry::list_all()` and compute `SyncStatus`: `Installed` + source exists → `Synced`; `Available` → `NotInstalled`; `Installed` + source missing → `Orphaned`.
9. **Create lib.rs** — Public module declarations, re-exports, doc inclusion from readme.md.
10. **Integrate into workspace** — Add `module/agent_inventory` to workspace `Cargo.toml` members + dependency table. Add row to `module/readme.md`. Update workspace `readme.md` Crates table and Architecture diagram (add agent_inventory at Layer 1).
11. **Integrate into dream** — Add `inventory` feature to `module/dream/Cargo.toml` gating `agent_inventory` dependency. Add `#[cfg(feature = "inventory")] pub mod inventory` to `module/dream/src/lib.rs`.
12. **Validate** — Run `cargo nextest run -p agent_inventory --all-features` and `cargo clippy -p agent_inventory --all-features -- -D warnings`. Then run `cargo nextest run -p dream --all-features` to verify re-export compiles.
13. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| Empty inventory (no adapters) | `Inventory::new()` | `list_all()` returns empty Vec |
| Single adapter, no assets | Claude Code adapter, empty `$PRO_CLAUDE` dir | Returns empty Vec, adapter name = `"claude_code"` |
| Single adapter, source + installed | Claude Code adapter, source dir + symlinks | Entries with `SyncStatus::Synced` |
| Source only, not installed | Source dir has skills, no symlinks | Entries with `SyncStatus::NotInstalled` |
| Orphaned symlink | Symlink exists, source deleted | Entries with `SyncStatus::Orphaned` |
| Kind filter | `list_by_kind(AssetKind::Skill)` | Only skill entries returned |
| Multiple adapters | Two adapters registered | `list_all()` merges results from both |
| AssetKind round-trip | All 6 variants | `as_str()` + `from_str()` identity |
| SyncStatus Display | All 3 variants | `format!()` produces `"✅ synced"`, `"⬇️ not installed"`, `"⚠️ orphaned"` |
| Missing env var | `$PRO_CLAUDE` unset, `$PRO` unset | Returns `InventoryError::EnvNotSet` |

## Acceptance Criteria

- `agent_inventory` crate compiles with `--all-features` and with no features
- `AgentAdapter` trait is object-safe (`Box<dyn AgentAdapter>` compiles)
- Claude Code adapter correctly maps all 6 `ArtifactKind` variants to `AssetKind`
- Claude Code adapter computes `SyncStatus` correctly (Synced/NotInstalled/Orphaned)
- `Inventory::list_all()` returns entries from all registered adapters
- `dream` re-exports `agent_inventory` under `inventory` feature flag
- All tests in `tests/` pass with `cargo nextest run -p agent_inventory --all-features`
- `cargo clippy -p agent_inventory --all-features -- -D warnings` clean

## Validation

### Checklist

Desired answer for every question is YES.

**Core types**
- [ ] Does `AssetEntry` have fields: agent, kind, name, status (SyncStatus)?
- [ ] Does `AssetKind` have 6 variants: Skill, Command, Rule, Agent, Plugin, Hook?
- [ ] Does `SyncStatus` have 3 variants: Synced, NotInstalled, Orphaned?
- [ ] Does `SyncStatus::fmt()` produce emoji-prefixed strings (✅/⬇️/⚠️)?
- [ ] Is `AgentAdapter` trait object-safe?
- [ ] Does `InventoryError` cover env-not-set and adapter errors?

**Claude Code adapter**
- [ ] Is adapter gated under `#[cfg(feature = "claude_code")]`?
- [ ] Does adapter use `claude_assets_core::registry::list_all()` internally?
- [ ] Does adapter map all 6 `ArtifactKind` variants correctly?
- [ ] Does `ClaudeCodeAdapter::name()` return `"claude_code"`?

**Integration**
- [ ] Is `agent_inventory` listed in workspace `Cargo.toml` members?
- [ ] Is `agent_inventory` in workspace dependency table?
- [ ] Does `dream` have `inventory` feature?
- [ ] Does `module/readme.md` include agent_inventory row?
- [ ] Does workspace `readme.md` include agent_inventory in Crates table and Architecture diagram?

**Quality**
- [ ] Does `cargo nextest run -p agent_inventory --all-features` pass?
- [ ] Does `cargo clippy -p agent_inventory --all-features -- -D warnings` pass?
- [ ] Are all public items documented with doc comments?
- [ ] Are all public functions annotated with `#[inline]`?

**Out of Scope confirmation**
- [ ] Are no CLI framework dependencies added (no unilang, no clap)?
- [ ] Does the crate have zero knowledge of consumer workspaces?

### Measurements

**M1 — Test suite passes**
Command: `cargo nextest run -p agent_inventory --all-features 2>&1 | tail -1`
Before: crate doesn't exist. Expected: `test result: ok`. Deviation: test failure.

**M2 — Clippy clean**
Command: `cargo clippy -p agent_inventory --all-features -- -D warnings 2>&1 | tail -3`
Before: crate doesn't exist. Expected: no warnings. Deviation: clippy warning.

**M3 — Workspace compiles**
Command: `cargo check --workspace 2>&1 | tail -1`
Before: compiles without agent_inventory. Expected: still compiles. Deviation: compilation error.

### Anti-faking checks

**AF1 — Adapter trait exists**
Check: `grep -c "trait AgentAdapter" module/agent_inventory/src/adapter.rs`
Expected: 1. Why: confirms trait definition exists, not just re-exports.

**AF2 — Claude Code feature gate**
Check: `grep -c 'cfg.*feature.*claude_code' module/agent_inventory/src/claude_code.rs`
Expected: ≥1. Why: confirms adapter is feature-gated, not unconditionally compiled.

**AF3 — dream re-export**
Check: `grep -c 'agent_inventory' module/dream/src/lib.rs`
Expected: ≥1. Why: confirms dream actually re-exports the new crate.

**AF4 — Workspace member registered**
Check: `grep -c 'agent_inventory' Cargo.toml`
Expected: ≥2. Why: must appear in both members list and dependency table.

## Outcomes

[Empty — populated upon task completion]
