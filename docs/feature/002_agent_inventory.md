# Feature: Agent Inventory

### Scope

- **Purpose**: Provide an agent-agnostic adapter layer for discovering code agent assets (skills, rules, commands, etc.) across multiple AI coding agents.
- **Responsibility**: Document the `agent_inventory` crate design, adapter trait, and consumer integration points.
- **In Scope**: Adapter trait, flat-table entry model, Claude Code adapter (via `claude_assets_core`), feature-gated adapters, kbase integration.
- **Out of Scope**: Individual agent CLIs (→ `claude_assets`), artifact installation (→ `claude_assets_core/install`), consumer workspace specifics (→ `integration/001_consumer_integration.md`).

### Design

**Purpose:** `agent_inventory` is a Layer 1 domain crate that normalizes asset discovery across different AI coding agents (Claude Code, Codex, Cursor, etc.) into a unified flat-table view. Each agent is represented by an adapter that implements the `AgentAdapter` trait; adapters are feature-gated so only required agents add compile-time cost.

**Architecture:**

```
agent_inventory (Layer 1, no CLI deps)
├── adapter.rs    — AgentAdapter trait definition
├── entry.rs      — AssetEntry (flat table row) + AssetKind enum
├── inventory.rs  — Inventory registry (collects adapters, merges results)
└── claude_code.rs — #[cfg(feature = "claude_code")] Claude Code adapter
                     uses claude_assets_core::registry + paths
```

**Core types:**

| Type | Responsibility |
|------|----------------|
| `AssetEntry` | Single flat-table row: agent name, asset kind, asset name, sync status |
| `AssetKind` | Enum: Skill, Command, Rule, Agent, Plugin, Hook |
| `SyncStatus` | Enum: Synced, NotInstalled, Orphaned — symlink sync state |
| `AgentAdapter` | Trait: `fn name(&self) -> &str`, `fn list(&self) -> Result<Vec<AssetEntry>>` |
| `Inventory` | Registry of adapters: `register()`, `list_all() -> Vec<AssetEntry>` |

**Sync status model:**

Since assets are installed via symlinks (live pointers), there is no version drift — only presence states:

| Emoji | Variant | Condition | Meaning |
|-------|---------|-----------|---------|
| ✅ | `Synced` | Source exists + symlink installed | Always in sync (symlink is live pointer) |
| ⬇️ | `NotInstalled` | Source exists, no symlink | Available in development dir, not yet installed |
| ⚠️ | `Orphaned` | Symlink exists, source missing | Installed but development source was deleted |

**AgentAdapter trait:**

```rust
pub trait AgentAdapter : core::fmt::Debug
{
  fn name( &self ) -> &str;
  fn list_all( &self ) -> Result< Vec< AssetEntry >, InventoryError >;
  fn list_by_kind( &self, kind : AssetKind ) -> Result< Vec< AssetEntry >, InventoryError >;
}
```

**Claude Code adapter:**

Feature-gated under `claude_code`. Wraps `claude_assets_core::registry::list_all()` and `AssetPaths::from_env()`. Maps `ArtifactKind` → `AssetKind` and `InstallStatus` → `SyncStatus`:

| `claude_assets_core` state | `SyncStatus` |
|---------------------------|--------------|
| `InstallStatus::Installed` + source exists | `Synced` |
| `InstallStatus::Available` | `NotInstalled` |
| `InstallStatus::Installed` + source missing | `Orphaned` |

**Consumer integration (kbase) — per-kind commands:**

Each asset kind is its own entity and gets a dedicated kbase command. Each command has its own feature gate.

**Phase 1 (current):** kbase uses `claude_assets_core` directly via cross-workspace path dependency. The `SyncStatus` logic and `AssetKind` filtering live in kbase's command module. This avoids blocking on the `agent_inventory` crate.

```toml
claude_assets_core = { path = "../../../../claude_tools/dev/module/claude_assets_core", optional = true }
```

**Phase 2 (future):** When `agent_inventory` is implemented, kbase migrates from `claude_assets_core` to `agent_inventory` — gaining multi-agent support with zero command API changes.

| kbase command | Asset kind | Feature gate | Collision notes |
|---------------|-----------|--------------|-----------------|
| `.skills` | Skill | `cmd_skills` | None |
| `.agent.commands` | Command | `cmd_agent_commands` | `.commands` reserved for future kbase use |
| `.agent.rules` | Rule | `cmd_agent_rules` | `.rules` exists (rulebook rule extraction) |
| `.agent.agents` | Agent | `cmd_agent_agents` | Awkward but consistent |
| `.hooks` | Hook | `cmd_hooks` | None |
| `.plugins` | Plugin | `cmd_plugins` | None |

Each command outputs an aligned flat table for its kind with emoji status:

```
kbase .skills

Agent        Name         Status
claude_code  commit       ✅ synced
claude_code  dev          ⬇️ not installed
claude_code  ops          ⬇️ not installed
claude_code  pr_review    ✅ synced
claude_code  old_skill    ⚠️ orphaned
```

Filterable with `agent::claude_code`, `status::synced`.

| Parameter | Values | Default |
|-----------|--------|---------|
| `agent::` | `claude_code` (more agents later) | all |
| `status::` | `synced`, `not_installed`, `orphaned` | all |
| `verbosity::` | 0 (table), 1 (table + paths) | 0 |

Implementation order: `.skills` first (primary use case), then remaining kinds as needed.

**Feature flags:**

| Feature | Activates | Dependency |
|---------|-----------|------------|
| `claude_code` | Claude Code adapter | `claude_assets_core` |
| (future) `codex` | Codex adapter | TBD |
| (future) `cursor` | Cursor adapter | TBD |

**Layer assignment:** Layer 1 (domain logic, zero CLI framework dependencies). No `unilang`, no `clap`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](001_workspace_design.md) | Workspace crate inventory (will include agent_inventory) |
| pattern | [pattern/001_crate_layering.md](../pattern/001_crate_layering.md) | Four-layer dependency hierarchy |
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Zero consumer workspace dependency rule |
| integration | [integration/001_consumer_integration.md](../integration/001_consumer_integration.md) | How kbase consumes agent_kit crates |
| source | `../../module/claude_assets_core/src/registry.rs` | Claude Code adapter wraps this registry |
| source | `../../module/claude_assets_core/src/artifact.rs` | ArtifactKind → AssetKind mapping source |
