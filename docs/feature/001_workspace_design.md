# Feature: Workspace Design

### Scope

- **Purpose**: Document the purpose, crate inventory, and scope of the agent_kit workspace.
- **Responsibility**: Describe what the workspace provides, what it excludes, and how the crates relate.
- **In Scope**: Workspace purpose, crate inventory (11 members), in-scope capabilities, out-of-scope boundaries, performance characteristics.
- **Out of Scope**: Crate layering pattern (→ `pattern/001_crate_layering.md`), privacy invariant (→ `invariant/001_privacy_invariant.md`), cross-workspace integration (→ `integration/001_willbe_integration.md`).

### Design

**Purpose:** agent_kit is a standalone Rust workspace for AI agent tooling: credential management, session storage, and process execution. The current implementation covers Claude Code in depth; the layered architecture is designed to expand to additional agents following the same crate pattern.

This workspace is a clean extraction from wtools. It has no knowledge of willbe's architecture, job queues, AI orchestration, or any private workspace concerns. It depends only on published wtools crates (error_tools, unilang, former) and the Rust standard library.

**Crate inventory:**

| Crate | Binary | Layer | Responsibility |
|-------|--------|-------|----------------|
| claude_storage_core | — | primitives | Parse Claude JSONL files: sessions, token statistics |
| claude_common | — | 0 | Shared domain primitives: ClaudePaths, process utilities |
| claude_profile_core | — | 1 | Token status + account domain logic (no CLI deps) |
| claude_manager_core | — | 1 | Version / settings_io / status domain helpers (no CLI deps) |
| claude_runner_core | — | 1 | Builder pattern for constructing and executing claude commands |
| claude_profile | clp | 2 | Manage Claude Code accounts, token status, and ~/.claude/ paths |
| claude_storage | cls | 2 | CLI for exploring Claude Code filesystem storage |
| claude_runner | clr | 2 | CLI for executing Claude Code with configurable parameters |
| agent_kit | — | 2 | Library facade re-exporting all Layer 0–1 core crates under feature-gated modules |
| claude_manager | clman | 2 | CLI for managing Claude Code installation, versions, and processes |
| claude_tools | clt | 3 | Super-app aggregator: all four Layer 2 crate commands in one binary |

**In scope:** Reading and parsing Claude Code's filesystem storage (`~/.claude/`); detecting sessions and continuation state; spawning `claude` with controlled parameters; managing Claude Code installation; managing accounts and active sessions; reading and writing Claude Code settings.

**Out of scope:** Job queue management, AI orchestration, wplan daemon integration, and any willbe-specific types (WorkDir, TopicName, JobId). If a feature requires knowing about queues, topics, or jobs, it does not belong here.

**Performance characteristics:**

`.status` command verbosity modes:

| Verbosity | Mode | Cost | Includes |
|-----------|------|------|----------|
| 0 | Fast (filesystem only) | O(P+S): ~50ms | Project count only |
| 1 | Fast (filesystem only) | O(P+S): ~50ms | Projects + session counts by type |
| 2–5 | Full (JSONL parsing) | O(total JSONL bytes): ~minutes | All above + entry counts + token usage |

`.list min_entries::N` reads every session file (O(total JSONL bytes)). With 1903 projects/2429 sessions/~7 GB of JSONL: cold cache ~12 minutes, warm cache ~25 seconds. Use `.count` instead for fast project/session counts.

`Session::count_entries()` uses byte-level string search (not full JSON parsing) on `"type":"user"` and `"type":"assistant"` patterns. Fast per-file, but O(total_JSONL_bytes) aggregate — avoid calling in loops over thousands of sessions without awareness of cost.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| pattern | [pattern/001_crate_layering.md](../pattern/001_crate_layering.md) | Four-layer dependency hierarchy between these crates |
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Zero willbe dependency rule |
| invariant | [invariant/004_performance.md](../invariant/004_performance.md) | Performance constraints for storage operations |
| integration | [integration/001_willbe_integration.md](../integration/001_willbe_integration.md) | How willbe consumes agent_kit crates |
| source | `../../Cargo.toml` | Workspace manifest and member declarations |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Purpose, Problem Statement, Workspace Structure, Crate Inventory, In Scope, Out of Scope, Performance Characteristics |
