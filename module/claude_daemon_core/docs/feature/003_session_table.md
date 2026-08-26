# Feature: Session Table

### Scope

- **Purpose**: Hold every session the daemon owns, addressable by a handle that survives the session being re-hosted under a new process.
- **In Scope**: `SessionTable`, `HostedSession`, `HostedSession::summary`, `Error::UnknownSession`.
- **Out of Scope**: Spawning the underlying process (→ `claude_pty_core`), deciding whether a session is busy (→ `claude_session_core`), the request shapes that reach the table (→ [002_wire_protocol.md](002_wire_protocol.md)).

### Behavior

`SessionTable` is a map from conversation id to `HostedSession`:

| Method | Contract |
|--------|----------|
| `new()` | Empty table |
| `len()` / `is_empty()` | Hosted-session count |
| `insert( session )` | Adds, replacing any entry with the same conversation id |
| `get_mut( session_id )` | `Error::UnknownSession` when absent |
| `remove( session_id )` | Returns the removed session; `Error::UnknownSession` when absent |
| `summaries()` | Every session as a `SessionSummary`, ordered by conversation id |

`summaries()` sorts rather than returning hash order so that repeated `list_sessions` calls against an unchanged table produce identical output. A list whose order changes between calls is unreadable in a terminal and impossible to diff in a test.

### `HostedSession`

| Field | Type | Purpose |
|-------|------|---------|
| `session_id` | `String` | The client-facing handle |
| `cwd` | `PathBuf` | Working directory the session runs in |
| `pty` | `PtySession` | The PTY-attached child, from `claude_pty_core` |
| `busy` | `bool` | Whether a turn is currently believed to be in flight |

`busy` is maintained by the daemon from `claude_session_core`'s `TurnWatcher`, not sampled from the registry directly — the difference matters, and is why [turn detection](../../../claude_session_core/docs/feature/002_turn_detection.md) is its own feature rather than a field read.

### Composition, Not Absorption

This crate holds the table and owns nothing below it:

| Concern | Owner |
|---------|-------|
| Allocating a PTY, spawning a child, queueing writes | `claude_pty_core` |
| Whether a PID is genuinely alive; whether a turn ended | `claude_session_core` |
| Which sessions exist and how clients address them | this crate |

The split is what keeps each piece testable on its own: the PTY layer can be exercised against `cat`, the session layer against a fixture directory, and the table against neither.

### Verification

```bash
cargo test -p claude_daemon_core --test table_test
```

`tests/table_test.rs` covers insert/replace by conversation id, `UnknownSession` on both lookup paths, and the stable ordering of `summaries()`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/table.rs` | `SessionTable` and `HostedSession` |
| source | `src/protocol.rs` | `SessionSummary`, what `summaries()` produces |
| doc | [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md) | Why the key is not the PID |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/table_test.rs` | Insert, lookup, removal, ordering |
