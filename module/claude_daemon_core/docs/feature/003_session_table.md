# Feature: Session Table

### Scope

- **Purpose**: Hold every session the daemon owns, addressable by a handle that survives the session being re-hosted under a new process.
- **In Scope**: `SessionTable`, `HostedSession`, `HostedSession::adopt`, `HostedSession::shutdown`, `Error::UnknownSession`, `Error::ReaderTaken`.
- **Out of Scope**: Spawning the underlying process (→ `claude_pty_core`), deciding whether a session is busy (→ `claude_session_core`), buffering the session's output (→ [004_session_output.md](004_session_output.md)), the request shapes that reach the table (→ [002_wire_protocol.md](002_wire_protocol.md)).

### Behavior

`SessionTable` is a map from conversation id to `HostedSession`:

| Method | Contract |
|--------|----------|
| `new()` | Empty table |
| `len()` / `is_empty()` | Hosted-session count |
| `insert( session )` | Adds, **returning** any entry it replaced |
| `get( session_id )` / `get_mut( session_id )` | `Error::UnknownSession` when absent |
| `remove( session_id )` | Returns the removed session; `Error::UnknownSession` when absent |
| `summaries()` | Every session as a `SessionSummary`, ordered by conversation id |
| `session_ids()` | Every conversation id, ordered |

`insert` is `#[ must_use ]` and hands back what it displaced rather than dropping it. A silently dropped `HostedSession` leaves a live child and a running pump thread with nobody holding the handle, so the caller has to decide what happens to it.

`summaries()` sorts rather than returning hash order so that repeated `list_sessions` calls against an unchanged table produce identical output. A list whose order changes between calls is unreadable in a terminal and impossible to diff in a test.

### `HostedSession`

Constructed with `HostedSession::adopt( session_id, cwd, pty )`, which takes the PTY's reader and starts an [output pump](004_session_output.md) on it. `Error::ReaderTaken` if something already took that reader — without it the master goes undrained, and an undrained master stalls its child as soon as the kernel's buffer fills.

| Accessor | Purpose |
|----------|---------|
| `session_id()` | The client-facing handle |
| `cwd()` | Working directory the session runs in |
| `pid()` | Current process id — diagnostic only |
| `busy()` / `set_busy()` | Whether a turn is currently believed to be in flight |
| `write( bytes )` | Deliver bytes to the session's terminal |
| `read_from( cursor )` | Output since `cursor`, as an `OutputSlice` |
| `resize( rows, cols )` | Change the terminal's dimensions |
| `summary()` | The `SessionSummary` for `list_sessions` |
| `shutdown()` | End the session and reap it |

`busy` is maintained by the daemon from `claude_session_core`'s `TurnWatcher`, not sampled from the registry directly — the difference matters, and is why [turn detection](../../../claude_session_core/docs/feature/002_turn_detection.md) is its own feature rather than a field read.

### Why the Fields Are Private

Two of them have an invariant between them. The pump holds a clone of the PTY master, and a session ends when the *last* master descriptor closes — so a session cannot be constructed without a pump draining it, nor torn down without stopping that pump first.

Public fields would make both mistakes expressible, and both are silent: the first presents as a session that stops responding under output, the second as a `shutdown` that never returns. Neither reports an error, because neither is one.

### Teardown Is an Ordered Ladder

`shutdown()` is three steps, each of which the next depends on:

| # | Step | Why it is here |
|---|------|----------------|
| 1 | Send `Ctrl-D` twice | An interactive program handed end-of-input exits through its own shutdown path — flushing a transcript, releasing locks. Nothing below gives it that chance. Twice, because canonical mode only reads `Ctrl-D` as end-of-input at the start of a line; sending a newline instead would submit whatever the user had half-typed |
| 2 | Wait up to 5s, then `SIGKILL` | A wedged child would otherwise hold the daemon here forever, because step 3 cannot proceed while it lives |
| 3 | Join the pump, then shut the PTY down | The pump releases its master only when its read ends, which happens when the child's descriptors close — which is what steps 1 and 2 exist to bring about |

Idempotent: a second call finds an already-exited child and returns the status the first recorded.

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
cd module/claude_daemon_core && ./verb/test
```

Or the suite directly, inside the container:

```bash
cargo nextest run -p claude_daemon_core --test table_test
```

`tests/table_test.rs` covers insert/replace by conversation id, `UnknownSession` on both lookup paths, the stable ordering of `summaries()`, and the teardown ladder against a real child blocked on stdin — the last one time-bounded, since a regression there hangs rather than fails.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/table.rs` | `SessionTable` and `HostedSession` |
| source | `src/output.rs` | The pump each session owns |
| source | `src/protocol.rs` | `SessionSummary`, what `summaries()` produces |
| doc | [004_session_output.md](004_session_output.md) | Why teardown has to stop the pump |
| doc | [invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md) | Why the key is not the PID |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/table_test.rs` | Insert, lookup, removal, ordering |
