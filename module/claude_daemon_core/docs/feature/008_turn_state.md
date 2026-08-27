# Feature: Turn State

### Scope

- **Purpose**: Report, per hosted session, whether a turn is currently in flight — accurately enough that a client can use it to decide an answer has finished.
- **In Scope**: `Daemon::with_background_reporting`, the refresh performed while answering `list_sessions`, and `SessionSummary::busy`.
- **Out of Scope**: How the status is detected from the registry (→ `claude_session_core`'s turn feature), how a client combines this with output silence (→ `claude_runner/docs/cli/command/14_chat.md`), the session table itself (→ [003_session_table.md](003_session_table.md)).

### Why This Exists

`HostedSession::busy` is what a client reads to decide whether to keep waiting. Nothing
in a session's output stream says "the answer is finished" — a terminal application emits
text continuously, including while it is thinking — so a flag maintained from outside the
stream is the only structural signal available.

### Where It Comes From

Claude Code writes its own status into its registry. `claude_session_core`'s `TurnWatcher`
turns a sequence of those statuses into transitions, and `busy` is set from the
transitions rather than sampled from the status directly.

The difference matters. A watcher is **edge-triggered**: it reports a boundary only on a
change *into* idle from a known-busy state, and reports nothing at all on first sighting.
A session that was already idle before anyone looked has no turn to have finished, and a
level-triggered read would call that a completed turn every time.

One watcher per session, keyed by conversation id. A single shared watcher fed by several
sessions would see their statuses interleaved and call every one of those a transition.

### When It Is Refreshed

While answering `list_sessions`, and at no other time.

The daemon has no timer to hang a refresh on. It is single-threaded and spends its life
blocked in `accept` — between requests there is no thread to sample on. Nor is there a
reason to: `busy` is only ever observed through the one request that reports it, so
refreshing there makes every answer exactly as fresh as the question. A client polling for
a turn boundary is itself the clock.

A scan that fails is silent. The registry is written by another program entirely, and a
directory that cannot be read means the daemon does not know whether anything changed —
which is precisely what leaving the last known state in place says. Failing the request
would take down a `list_sessions` that has a perfectly good answer to every other part of
the question.

### The Guarantee `idle` Needs

An observed `idle` means "the turn is over" **only** when the session was started with
`CLAUDE_CODE_BG_TASKS_REPORT_RUNNING=1`. Without it, a session parked on an outstanding
background task reports `idle` too, and nothing distinguishes the two afterwards.

This crate cannot know, because it does not start the session — `spawner` does, and
`spawner` belongs to whoever builds a daemon out of this. So the claim is theirs to make:

```rust,ignore
use claude_daemon_core::{ BackgroundReporting, Daemon, BG_TASKS_REPORT_RUNNING_ENV };

// The spawner sets the variable …
let spawner = | cwd : &std::path::Path |
{
  let config = claude_pty_core::SessionConfig::new( "claude" )
    .cwd( cwd )
    .env( BG_TASKS_REPORT_RUNNING_ENV, "1" );
  claude_pty_core::PtySession::spawn( &config ).map_err( claude_daemon_core::Error::Pty )
};

// … so the daemon may claim it.
let daemon = Daemon::new( sessions_dir, spawner )
  .with_background_reporting( BackgroundReporting::Enabled );
```

The default is `BackgroundReporting::Unknown`, which is the conservative answer and the
only one this crate can give on a caller's behalf. Claiming `Enabled` falsely makes `busy`
go false while a background task is still outstanding — exactly the failure the flag
exists to describe.

`TurnEvent::SettledUnverified` — a turn boundary observed without the guarantee — is
reported as `busy = false`, the same as a verified one. There is nothing else a boolean
can say, and the distinction is not lost: it is the `reporting` value the caller set. What
must not happen is a session stuck at `busy` forever because the only honest answer was
"probably".

### Verification

```bash
cargo test -p claude_daemon_core --test serve_test
```

On a live daemon, the flag is what the `STATE` column reads from:

```bash
clr chat "count slowly to twenty" &   # a turn that lasts long enough to catch
clr sessions                          # STATE reads busy
sleep 30
clr sessions                          # STATE reads idle
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/serve.rs` | `Daemon::refresh_turns`, `with_background_reporting` |
| source | `src/table.rs` | `HostedSession::busy` / `set_busy` |
| doc | [003_session_table.md](003_session_table.md) | The table the flag lives in |
| doc | [006_serving_clients.md](006_serving_clients.md) | The `list_sessions` request that reports it |
| doc | [api/001_daemon_surface.md](../api/001_daemon_surface.md) | Full signature contract |
| test | `tests/serve_test.rs` | Dispatch against a real socket and real children |
