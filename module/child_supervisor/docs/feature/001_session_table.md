# Feature: Session Table

### Scope

- **Purpose**: Hold every child process a caller is supervising, addressable by a handle the caller controls rather than one this crate invents.
- **In Scope**: `SessionTable`, `HostedSession`, `HostedSession::adopt`, `HostedSession::shutdown`, `Error::UnknownSession`, `Error::ReaderTaken`.
- **Out of Scope**: Spawning the underlying process (→ `claude_pty_core`), deciding whether a session is busy — this crate only stores the flag a caller sets (→ the caller; `claude_daemon_core` drives it from `claude_session_core`'s `TurnWatcher`), buffering the session's output (→ [002_session_output.md](002_session_output.md)), building a wire-facing snapshot from the table (→ `claude_daemon_core`, which owns that shape).

### Behavior

`SessionTable` is a map from a caller-chosen id to `HostedSession`:

| Method | Contract |
|--------|----------|
| `new()` | Empty table |
| `len()` / `is_empty()` | Hosted-session count |
| `insert( session )` | Adds, **returning** any entry it replaced |
| `get( session_id )` / `get_mut( session_id )` | `Error::UnknownSession` when absent |
| `remove( session_id )` | Returns the removed session; `Error::UnknownSession` when absent |
| `session_ids()` | Every id, ordered |
| `take_exited()` | Removes and returns every session whose child has already exited, ordered by id |

`insert` is `#[ must_use ]` and hands back what it displaced rather than dropping it. A silently dropped `HostedSession` leaves a live child and a running pump thread with nobody holding the handle, so the caller has to decide what happens to it.

`session_ids()` sorts rather than returning hash order so that repeated listings against an unchanged table produce identical output. A list whose order changes between calls is unreadable in a terminal and impossible to diff in a test.

### `HostedSession`

Constructed with `HostedSession::adopt( session_id, cwd, pty )`, which takes the PTY's reader and starts an [output pump](002_session_output.md) on it. `Error::ReaderTaken` if something already took that reader — without it the master goes undrained, and an undrained master stalls its child as soon as the kernel's buffer fills.

| Accessor | Purpose |
|----------|---------|
| `session_id()` | The caller-facing handle |
| `cwd()` | Working directory the session runs in |
| `pid()` | Current process id — diagnostic only |
| `busy()` / `set_busy()` | A caller-maintained flag; this crate stores it and enforces nothing about when it changes |
| `write( bytes )` | Deliver bytes to the session's terminal |
| `read_from( cursor )` | Output since `cursor`, as an `OutputSlice` |
| `resize( rows, cols )` | Change the terminal's dimensions |
| `exited()` | `Some( status )` once the child has exited, without waiting for it |
| `shutdown()` | End the session and reap it |

This crate has no opinion on what `busy` should mean or when it should flip — it is a caller-set bit alongside the handle. `claude_daemon_core` is the one caller today, and it drives the flag from `claude_session_core`'s `TurnWatcher`.

### Why the Fields Are Private

Two of them have an invariant between them. The pump holds a clone of the PTY master, and a session ends when the *last* master descriptor closes — so a session cannot be constructed without a pump draining it, nor torn down without stopping that pump first.

Public fields would make both mistakes expressible, and both are silent: the first presents as a session that stops responding under output, the second as a `shutdown` that never returns. Neither reports an error, because neither is one.

### Teardown Is an Ordered Ladder

`shutdown()` is three steps, each of which the next depends on:

| # | Step | Why it is here |
|---|------|----------------|
| 1 | Send `Ctrl-D` twice | An interactive program handed end-of-input exits through its own shutdown path — flushing a transcript, releasing locks. Nothing below gives it that chance. Twice, because canonical mode only reads `Ctrl-D` as end-of-input at the start of a line; sending a newline instead would submit whatever the user had half-typed |
| 2 | Wait up to 5s, then `SIGKILL` | A wedged child would otherwise hold the caller here forever, because step 3 cannot proceed while it lives |
| 3 | Join the pump, then shut the PTY down | The pump releases its master only when its read ends, which happens when the child's descriptors close — which is what steps 1 and 2 exist to bring about |

Idempotent: a second call finds an already-exited child and returns the status the first recorded.

### Composition, Not Absorption

This crate holds the table and owns nothing below or above it:

| Concern | Owner |
|---------|-------|
| Allocating a PTY, spawning a child, queueing writes | `claude_pty_core` |
| Whether a PID is genuinely alive; whether a turn ended | Whatever the caller uses — `claude_session_core`, in `claude_daemon_core`'s case |
| Which sessions exist and how a caller addresses them | this crate |
| Naming a session, and building a wire-facing snapshot of one | the caller — see `claude_daemon_core/docs/feature/002_wire_protocol.md` for `SessionSummary` and `claude_daemon_core/docs/feature/006_serving_clients.md` for where it is assembled |

The split is what keeps each piece testable on its own: the PTY layer can be exercised against `cat`, whatever tracks liveness against a fixture directory, and the table against neither.

### Verification

```bash
cd module/child_supervisor && cargo nextest run --test table_test
```

`tests/table_test.rs` covers insert/replace by id, `UnknownSession` on both lookup paths, the stable ordering of `session_ids()`, reaping an exited child without disturbing a live one, and the teardown ladder against a real child blocked on stdin — the last one time-bounded, since a regression there hangs rather than fails.

Whether a `SessionSummary` built from this table's accessors matches a hosted session end to end (session_id, pid, cwd, busy) is covered where that DTO is defined: `claude_daemon_core`'s `tests/serve_test.rs` (`srv03`), against the real `Request::ListSessions` response.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/table.rs` | `SessionTable` and `HostedSession` |
| source | `src/output.rs` | The pump each session owns |
| doc | [002_session_output.md](002_session_output.md) | Why teardown has to stop the pump |
| doc | `claude_daemon_core/docs/invariant/002_conversation_id_key.md` | Why the key is not the PID — the Claude-specific rationale, unchanged by this crate's genericization |
| doc | `claude_daemon_core/docs/api/001_daemon_surface.md` | Full signature contract, as re-exported by `claude_daemon_core` |
| test | `tests/table_test.rs` | Insert, lookup, removal, ordering, reaping |
