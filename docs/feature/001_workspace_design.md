# Feature: Workspace Design

### Scope

- **Purpose**: Document the purpose, crate inventory, and scope of the assistant workspace.
- **Responsibility**: Describe what the workspace provides, what it excludes, and how the crates relate.
- **In Scope**: Workspace purpose, crate inventory (19 members), in-scope capabilities, out-of-scope boundaries, performance characteristics.
- **Out of Scope**: Crate layering pattern (→ `pattern/001_crate_layering.md`), privacy invariant (→ `invariant/001_privacy_invariant.md`), cross-workspace integration (→ `integration/001_consumer_integration.md`).

### Design

**Purpose:** assistant is a standalone Rust workspace for coding agent tooling: credential management, session storage, and process execution. The current implementation covers Claude Code in depth; the layered architecture is designed to expand to additional coding agents following the same crate pattern.

This workspace is self-contained and has no knowledge of consumer workspace architecture, job queues, AI orchestration, or any private workspace concerns. It depends only on published companion crates (`error_tools`, `unilang`, `data_fmt`, `cli_fmt`, and others) and the Rust standard library.

**Crate inventory:**

| Crate | Binary | Layer | Responsibility |
|-------|--------|-------|----------------|
| claude_storage_core | — | primitives | Parse Claude JSONL files: sessions, token statistics |
| claude_auth | — | primitives | Anthropic OAuth token refresh transport; `TokenRefreshResult`, `AuthError` |
| claude_quota | — | primitives | Anthropic API rate-limit HTTP transport; `RateLimitData`, `QuotaError` |
| claude_journal | — | primitives | Append-only event journal library: JSONL writer, reader, event types |
| claude_core | — | 0 | Shared domain primitives: ClaudePaths, process utilities |
| claude_profile_core | — | 1 | Token status + account domain logic (no CLI deps) |
| claude_version_core | — | 1 | Version / settings_io / status domain helpers (no CLI deps) |
| claude_runner_core | — | 1 | Builder pattern for constructing and executing claude commands |
| claude_assets_core | — | 1 | Symlink-based artifact installer domain logic (no CLI deps) |
| claude_profile | clp / claude_profile | 2 | Manage Claude Code accounts, token status, and ~/.claude/ paths |
| claude_storage | clg / claude_storage | 2 | CLI for exploring Claude Code filesystem storage |
| claude_journal_viewer | clj | 2 | CLI and web viewer for CLR journal events |
| claude_runner | clr / claude_runner | 2 | CLI for executing Claude Code with configurable parameters |
| dream | — | 2 | Agent-agnostic library facade re-exporting all core crates (Layer 0, *, 1) under feature-gated modules |
| claude_version | clv / claude_version | 2 | Claude Code version manager CLI |
| claude_assets | cla / claude_assets | 2 | CLI for installing Claude Code artifacts (rules, skills, commands) via symlinks |
| assistant | ast / assistant | 3 | Agent-agnostic super-app aggregator: all Layer 2 CLI crates in one binary |
| assistant_kit | — | 3 | Library facade for the five Claude Code Layer 2 CLI crates (excludes journal viewer) |
| runbox | crb / runbox | * | Scaffold container runner integration files into a project |

**Binaries** (16 targets — 7 crates expose both canonical name and short alias; `claude_runner` additionally exposes alias `c`; `claude_journal_viewer` exposes alias `clj` only):

| Binary | Crate | Kind | Entry point |
|--------|-------|------|-------------|
| `cla` | `claude_assets` | alias | `src/bin/cla.rs` |
| `claude_assets` | `claude_assets` | canonical | `src/main.rs` |
| `clg` | `claude_storage` | alias | `src/bin/clg.rs` |
| `claude_storage` | `claude_storage` | canonical | `src/main.rs` |
| `clj` | `claude_journal_viewer` | primary | `src/cli_main.rs` |
| `clv` | `claude_version` | alias | `src/bin/clv.rs` |
| `claude_version` | `claude_version` | canonical | `src/main.rs` |
| `clp` | `claude_profile` | alias | `src/bin/clp.rs` |
| `claude_profile` | `claude_profile` | canonical | `src/main.rs` |
| `clr` | `claude_runner` | alias | `src/bin/clr.rs` |
| `c` | `claude_runner` | alias | `src/bin/c.rs` |
| `claude_runner` | `claude_runner` | canonical | `src/main.rs` |
| `ast` | `assistant` | alias | `src/bin/ast.rs` |
| `assistant` | `assistant` | canonical | `src/main.rs` |
| `crb` | `runbox` | alias | `src/bin/crb.rs` |
| `runbox` | `runbox` | canonical | `src/main.rs` |

**Naming convention:** Crates prefixed `claude_*` are Claude Code-specific. `dream`, `assistant`, and `runbox` are intentionally unprefixed — `dream` and `assistant` form the agent-agnostic integration layer; `runbox` is a standalone project scaffolding tool not specific to any agent.

**In scope:** Reading and parsing Claude Code's filesystem storage (`~/.claude/`); detecting sessions and continuation state; spawning `claude` with controlled parameters; managing Claude Code installation; managing accounts and active sessions; reading and writing Claude Code settings.

**Out of scope:** Job queue management, AI orchestration, orchestration daemon integration, and any consumer-workspace-specific types (WorkDir, TopicName, JobId). If a feature requires knowing about queues, topics, or jobs, it does not belong here.

**Performance characteristics:** See [invariant/004_performance.md](../invariant/004_performance.md) for the fast-path vs full-parse cost model, concrete measurements, and avoidance rules.

### Integrations

| File | Relationship |
|------|--------------|
| [integration/001_consumer_integration.md](../integration/001_consumer_integration.md) | How consumer workspaces consume assistant crates |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Zero consumer workspace dependency rule |
| [invariant/002_versioning_strategy.md](../invariant/002_versioning_strategy.md) | Shared workspace version policy |
| [invariant/003_testing_strategy.md](../invariant/003_testing_strategy.md) | TDD baseline and Level 3 enforcement |
| [invariant/004_performance.md](../invariant/004_performance.md) | Performance constraints for storage operations |
| [invariant/005_dependency_management.md](../invariant/005_dependency_management.md) | Workspace dep centralization policy |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_crate_layering.md](../pattern/001_crate_layering.md) | Four-layer dependency hierarchy between these crates |

### Sources

| File | Relationship |
|------|--------------|
| `../../Cargo.toml` | Workspace manifest and member declarations |

### Provenance

| File | Relationship |
|------|--------------|
| `spec.md` (deleted — migrated here) | Purpose, Problem Statement, Workspace Structure, Crate Inventory, In Scope, Out of Scope, Performance Characteristics |
