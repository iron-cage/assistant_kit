# Data Structure: Storage Hierarchy

### Scope

- **Purpose**: Model Claude Code's filesystem-native conversation storage as a four-level hierarchy of typed Rust values.
- **Responsibility**: Documents the Storage → Project → Session → Entry model: roles, relationships, and invariants.
- **In Scope**: Type responsibilities, lazy-loading semantics, project-type distinction (UUID vs path), entry content variants.
- **Out of Scope**: Path encoding/decoding (→ `algorithm/001_path_encoding.md`), filter types (→ `data_structure/002_filter_types.md`).

### Abstract

The hierarchy mirrors the `~/.claude/` directory layout exactly. Each level is a typed value that can be independently loaded, enumerated, or queried. Lazy loading ensures that reading a list of projects does not parse any JSONL files until sessions are explicitly requested.

### Structure

**Storage** — entry point wrapping `~/.claude/`.
- Enumerates all project directories.
- Provides global statistics across all sessions.
- Path: `~/.claude/` root.

**Project** — directory containing one or more session files.
- Two variants: UUID-based (web/IDE sessions, e.g. `{uuid}/`) and path-based (CLI sessions, e.g. `-home-user-project/`).
- Contains the decoded filesystem path for path-based projects.
- Holds a list of session file paths (not yet loaded on construction).

**Session** — a single JSONL conversation file.
- Identity: session ID string (file stem).
- Entries are lazy-loaded on first access.
- Supports append-only write via `append_entry()`.
- Statistics (entry count, token totals, timestamps) available without full entry parse.

**Entry** — one message in the conversation.
- Role: user or assistant.
- Content blocks: text, thinking, tool_use, tool_result.
- Metadata: timestamp, cwd, git branch, token usage.
- Non-conversation entries (queue-operation, summary, file-history-snapshot) are silently skipped during loading.

**Statistics types** — aggregated metrics.
- `SessionStats`: per-session totals (entries, input/output tokens, first/last timestamps).
- `ProjectStats`: per-project aggregation over sessions.
- `GlobalStats`: workspace-wide summary with per-project breakdown.

### Operations

- **Enumerate projects**: `Storage::list_projects()`, `list_projects_filtered(&ProjectFilter)`.
- **Load sessions**: `Project::sessions()`, `sessions_filtered(&SessionFilter)`.
- **Read entries**: `Session::entries()` (full parse), `count_entries()` (fast byte-level count).
- **Append entry**: `Session::append_entry(&Entry)` — atomic via temp-file rename.
- **Statistics**: `Session::stats()`, `Project::stats()`, `Storage::global_stats()`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/storage.rs` | Storage type implementation |
| source | `../../src/project.rs` | Project type, UUID/path variant handling |
| source | `../../src/session.rs` | Session lazy-load, count_entries(), append_entry() |
| source | `../../src/entry.rs` | Entry type, content block variants |
| source | `../../src/stats.rs` | SessionStats, ProjectStats, GlobalStats |
| doc | `../algorithm/001_path_encoding.md` | Path encoding for path-based project names |
| doc | `../invariant/001_safety_guarantees.md` | Append-only and atomic write invariants |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; storage model and core types sections extracted here |
