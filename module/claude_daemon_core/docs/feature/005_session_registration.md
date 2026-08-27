# Feature: Session Registration

### Scope

- **Purpose**: Learn the conversation id of a session the daemon has just spawned, so it can be filed under the handle clients will use.
- **In Scope**: `await_session_id`, `registration::lookup`, `REGISTRATION_TIMEOUT`, `Error::NoRegistration`.
- **Out of Scope**: Parsing registry files (→ `claude_session_core`), spawning the process (→ `claude_pty_core`), what the id is then used for (→ [003_session_table.md](003_session_table.md)).

### Why This Is a Wait, Not a Return Value

Nothing the daemon does assigns a conversation id. Claude Code mints its own and publishes it by writing a record into its session registry — which happens some milliseconds *after* the process starts, and therefore after the spawn call has already returned.

So the id cannot be an output of spawning. It can only be observed afterwards, which leaves a window in which the daemon holds a live session it cannot yet name. This feature is what closes that window.

### Behavior

```
await_session_id( sessions_dir, pid, timeout, alive ) -> Result< String >
```

| Outcome | Condition |
|---------|-----------|
| `Ok( id )` | A record naming `pid` was found |
| `Err( NoRegistration )` | `alive()` reported the child gone, or `timeout` elapsed |
| `Err( Registry )` | The registry directory exists but cannot be read |

The loop scans, then decides whether to keep waiting, then sleeps. `REGISTRATION_TIMEOUT` is 30 seconds — registration is one small file write, so the normal case is well under a second, and the margin is for a cold start on a loaded machine rather than for a process that is never going to register.

A *missing* registry directory is not an error. The first session on a machine runs before anything has created it, and an empty scan is the correct answer.

### Why `alive` Is a Parameter

The registry cannot distinguish "has not registered yet" from "will never register". Only the caller — which holds the child handle — can tell the difference.

Without it, a child that dies during startup costs the full timeout before the daemon reports a failure it could have reported at once. The client is waiting on that.

### Scan Before Liveness

The order of those two checks is load-bearing. A short-lived session can register and exit before the first poll comes round; its record is on disk and correct. Consulting liveness first would discard a conversation id that is sitting readable, over a race the caller cannot influence.

So the scan always runs first, and its result always wins.

### Matching on a PID, Once

Everywhere else this crate refuses to key on a PID, for the reason `claude_session_core`'s liveness invariant documents: a PID number outlives the process that held it, so it names a process only within a known incarnation.

This is the one place the incarnation *is* known. The daemon spawned the child itself and holds its handle, so the number cannot have been recycled while the caller is still looking at it. The PID answers one question, once — "which of these records is my child?" — and is then discarded in favour of the conversation id.

### Torn Writes

Claude Code rewrites registry files in place, so a scan can catch one half-written. An unparseable file is skipped rather than failing the lookup: failing the whole scan over one would make registration flaky in exactly the busy conditions where sessions are being created.

### A Child That Never Registers Is Killed

Giving up on the wait leaves a live process nobody has a name for, and ending it is the
caller's job — `Daemon::spawn` does it before returning the error, via `end_unregistered`.

Nothing does it otherwise. `PtySession` has no `Drop`, and `std::process::Child`
deliberately does not kill on drop either, so the natural-looking outcome — return the
error, let the handle fall out of scope — reparents the child to init, where it holds its
terminal for the life of the machine and is addressable by nobody: the session table never
had it, so no `shutdown` can reach it and no `list_sessions` will admit it exists.

`kill` comes before `shutdown`, which inverts the usual preference for asking politely. A
child that failed to register has no conversation to flush, so there is nothing the polite
form would preserve — and this daemon is single-threaded, so one child that ignores the
hangup would freeze every other session behind it.

### Verification

```bash
cd module/claude_daemon_core && ./verb/test
```

`tests/serve_test.rs`'s srv12 covers the kill: it spawns a child that ignores `SIGHUP` and
never reads its terminal, so it survives every incidental teardown and only the explicit
kill can end it. That detail is the test — with an ordinary `cat`, closing the master end
kills the child for free and srv12 passes with the fix removed.

Or the suite directly, inside the container:

```bash
cargo nextest run -p claude_daemon_core --test registration_test
```

`tests/registration_test.rs` covers matching, the missing directory, both give-up paths (early on a dead child, at the deadline on a live one), a record that lands mid-wait, the scan-before-liveness ordering, and a torn file beside a good one.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/registration.rs` | `await_session_id`, `lookup` |
| source | `src/serve.rs` | `Daemon::spawn`, `end_unregistered` |
| source | `src/error.rs` | `Error::NoRegistration`, `Error::Registry` |
| doc | [../invariant/002_conversation_id_key.md](../invariant/002_conversation_id_key.md) | Why the id, not the PID, is the handle |
| doc | [../../../claude_session_core/docs/feature/001_registry_scan.md](../../../claude_session_core/docs/feature/001_registry_scan.md) | The registry format and how it is read |
| test | `tests/registration_test.rs` | Matching, waiting, giving up, torn files |
| test | `tests/serve_test.rs` | srv12 — the child that never registers does not outlive the failure |
