# Feature: Registry Scan

### Scope

- **Purpose**: Answer "which Claude Code sessions are running right now, and where" by reading the registry Claude Code maintains for its own use.
- **In Scope**: `scan`, `scan_live`, `SessionRecord`, `SessionRecord::parse`, `SessionRecord::is_alive`, `SessionStatus`.
- **Out of Scope**: Turn boundaries (→ [002_turn_detection.md](002_turn_detection.md)), the liveness rule itself (→ [invariant/001_liveness_four_clauses.md](../invariant/001_liveness_four_clauses.md)), conversation content (→ `claude_storage_core`).

### The Registry

Claude Code writes one JSON file per running process into its sessions directory, named by PID. Each file carries the process's identity and self-reported status.

| Field | Type on disk | Meaning |
|-------|--------------|---------|
| `pid` | number | Process id |
| `sessionId` | string | Conversation id — the join key to the transcript |
| `cwd` | string | Working directory |
| `procStart` | **string** | Process start time in clock ticks since boot |
| `version` | string | Claude Code version |
| `kind` | string | e.g. `interactive` |
| `entrypoint` | string | e.g. `cli` |
| `name` | string | Human-readable session name |
| `status` | string | `busy`, `idle`, or another value |
| `updatedAt` | number | Milliseconds since the Unix epoch |

`procStart` is stored as a JSON string, not a number. A reader that calls a numeric accessor on it gets `None` and silently loses the one field that makes PID-based identity trustworthy — which is why `SessionRecord::parse` reads it with `get_str` and then parses the digits.

### Why This Crate and Not `claude_storage_core`

`claude_storage_core` already reads Claude Code's on-disk state, so the reasonable first question is why the registry is not read there too. The two stores have opposite properties:

| | Transcripts (`claude_storage_core`) | Registry (this crate) |
|---|---|---|
| Lifetime | Permanent | Ephemeral — deleted on exit |
| Mutation | Append-only | Rewritten in place |
| Keyed by | Conversation id | PID |
| Answers | "what was said" | "what is running" |
| Needs `/proc` | No | Yes |

`claude_storage_core` holds a zero-runtime-dependency guarantee and a purely-file-parsing model. Reading the registry usefully requires consulting `/proc` to tell a live record from a stale one, which is a different kind of operation on a different kind of data. Keeping them apart is what lets `claude_storage_core` stay a parser.

This crate does depend on `claude_storage_core` — for its hand-rolled JSON parser, so a third JSON implementation is not introduced.

### Scan Semantics

`scan( sessions_dir )` reads every `*.json` in the directory and returns the records that parsed, sorted by PID.

- **A missing directory is an empty result, not an error.** Claude Code creates the directory when it first runs; a machine where it has never run is a valid state with zero sessions, not a failure.
- **An unparseable file is skipped, not fatal.** These files are rewritten in place, so a reader can observe a torn write. One torn file must not blank the whole scan.
- **`pid` and `sessionId` are required.** A record missing either cannot be used for anything and is dropped.

`scan_live( sessions_dir )` is `scan` filtered by `SessionRecord::is_alive` — the records whose process is still running *and* is the same incarnation that wrote the file.

### Verification

```bash
# Records Claude Code currently has on disk, live or not:
ls "${CLAUDE_HOME:-$HOME/.claude}/sessions/"

# What the crate makes of them:
cargo test -p claude_session_core --test registry_test
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/registry.rs` | `scan`, `scan_live`, `SessionRecord`, `SessionStatus` |
| source | `src/liveness.rs` | The `/proc` checks behind `is_alive` |
| doc | [invariant/001_liveness_four_clauses.md](../invariant/001_liveness_four_clauses.md) | Why liveness needs four clauses |
| doc | [api/001_session_surface.md](../api/001_session_surface.md) | Full signature contract |
| test | `tests/registry_test.rs` | Parsing, missing-directory, and torn-write handling |
