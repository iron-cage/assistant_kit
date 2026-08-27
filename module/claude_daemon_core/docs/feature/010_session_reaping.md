# Feature: Session Reaping

### Scope

- **Purpose**: Release a hosted session that nobody has used for long enough, and end the daemon itself once it has nothing left to host — so neither a session nor the process holding it outlives its usefulness.
- **In Scope**: What counts as idle, the clock the daemon acquires in order to notice, the linger period before an empty daemon exits, and the bound on how much reaping one tick may do.
- **Out of Scope**: Re-opening a released conversation (→ [009_session_resume.md](009_session_resume.md)), releasing a session on purpose for an interactive client (→ `claude_runner/docs/feature/008_interactive_handoff.md`), how a turn is detected (→ [008_turn_state.md](008_turn_state.md)).

### Why This Exists

Nothing currently ends a hosted session. A session lives until `clr daemon stop` takes the
daemon and every session with it, or until the machine reboots — each one holding a `claude`
process, a pseudo-terminal, an output pump thread, and a turn watcher, indefinitely, for a
conversation that may have been abandoned days ago.

### The Daemon Has No Clock

This is the structural obstacle, and it is worth stating before the policy, because it rules
out the obvious implementation.

The main loop is `serve_once` → check the stop flag → repeat, and `serve_once` blocks in
`accept` with no timeout. [008_turn_state.md](008_turn_state.md) already records the
consequence and treats it as a virtue: *"between requests there is no thread to sample on …
a client polling for a turn boundary is itself the clock."*

That works for turn state, which is only ever *observed through* a request. It does not work
here. Both timeouts in this feature must fire **precisely when no client is talking**, because
that is the definition of idle. A check that runs on each incoming request is structurally
blind to the only condition it exists to detect.

So the daemon needs a clock, and there are three ways to give it one:

| | Mechanism | Verdict |
|---|---|---|
| A | Non-blocking `accept` plus a sleep loop | Rejected — adds up to one tick of latency to *every* command, including `clr sessions`. |
| B | `poll(2)` or `SO_RCVTIMEO` on the listener fd | Rejected — correct and free at runtime, but needs `unsafe` FFI, which the workspace confines to `claude_pty_core`. `poll` has nothing to do with pseudo-terminals. |
| C | **A waker thread acting as a synthetic client** | Chosen. |

**C is not a workaround; it is the existing idiom.** The daemon already treats an arriving
connection as its clock. A waker is a client that wants nothing: it sleeps for one tick,
connects to the socket, and hangs up. `accept` returns, the loop turns over, the reaper runs.

The decisive property is that every piece of daemon state stays single-threaded and owned by
the main loop. A genuine reaper thread would need a lock around the session table and would
undo the simplification the whole crate is built on.

Reaping is therefore driven from the loop, next to the stop check, so that **any** connection
drives it and the waker only guarantees a floor rate:

```rust,ignore
loop
{
  if let Err( error ) = serve_once( &listener, &mut daemon ) { /* log */ }
  daemon.reap();
  if daemon.stop_requested() || daemon.should_exit() { break; }
}
```

### Idle Means Two Things

A session running a forty-minute autonomous task has **zero client activity**. Under a naive
"no `send` for N minutes" rule it gets killed mid-turn, and the answer is lost.

So a session is idle only when both hold:

1. Nothing has touched it for `idle_timeout` — `send`, `read`, `resize`, or its own spawn.
2. It is not busy, per [008_turn_state.md](008_turn_state.md).

Which forces a consequence that is easy to miss and expensive to get wrong: **`refresh_turns`
must run inside the reaper**, before any decision. It currently runs only while answering
`list_sessions`. A reaper that skipped it would read a `busy` flag last updated whenever a
client happened to ask — which, on an idle daemon, is exactly never — and would kill a
working session on stale state.

`last_active` is therefore bumped on client activity *and* on every observed `busy`.

### Linger

The second clock is the daemon's own and starts empty-handed: when the last session is
released, `empty_since` is stamped. If `linger` elapses with the table still empty, the daemon
exits through its ordinary shutdown path — socket unlinked before the lock is dropped, so
there is no window in which the lock is free while a socket still invites connections.

Any spawn clears the stamp. The clock measures *continuously* empty, not cumulatively.

**Exiting is the default, and the two clocks compound.** The linger clock cannot start until
the last session has already been reaped, so the shortest path from a final command to a dead
daemon is:

> last command → `idle_timeout` (30 min) → session released → `linger` (5 min) → daemon exits

**35 minutes of continuous inactivity**, not five. That margin is what makes a short linger
safe: no ordinary working rhythm reaches it, and the cost of being wrong is a cold start on
the next `clr chat` — a few seconds, with the conversation intact via
[009_session_resume.md](009_session_resume.md) — rather than anything lost.

### Settled Defaults

| Setting | Default | `0` |
|---|---|---|
| `idle_timeout` | 30 min | never reap a session |
| `linger` | 5 min | never exit the daemon |
| `tick` | 30 s | — |

All three are injectable, following `with_registration_timeout`'s precedent, because no test
can wait out a thirty-minute default.

### One Session Per Tick

`HostedSession::shutdown` waits up to `SHUTDOWN_GRACE` (5s) for a child to exit on its own.
The daemon is single-threaded, so that wait is the whole daemon. Reaping four sessions in one
tick is twenty seconds during which no client is served, which reads from outside as a hung
daemon.

The reaper therefore releases **at most one session per tick**. With a tick far shorter than
`idle_timeout`, a backlog drains long before anyone notices it existed.

### Released, Not Destroyed

A reaped session's conversation is re-openable — that is [009_session_resume.md](009_session_resume.md),
and it is a hard prerequisite rather than a companion feature. Reaping without resume silently
destroys conversations; the argument is set out in that document and not repeated here.

This inverts the feature's character, and the defaults should be read in that light. *"Your
session is killed after thirty minutes"* would be a reason to set the timeout high. *"Your
session sleeps and wakes where it left off"* is not.

### What Is Not Yet Settled

- **Where they are configured.** The `CLR_*` environment tier and the TOML config tier both
  already exist; which one owns these is **TBD**. The values themselves are settled above.

### Adjacent Defect This Closes

The tick is also the first place anything periodically checks whether a hosted child is still
alive. `PtySession::try_wait` exists and nothing calls it on a schedule, so a session whose
child died stays listed as hosted indefinitely and fails only when a client tries to `send` to
it. Reaping dead children costs one extra condition in a loop that now exists anyway.

### Verification

```bash
cargo test -p claude_daemon_core --test serve_test
```

Against a live daemon, with short timeouts injected:

```bash
clr chat "hello"
clr sessions                 # one row

# Wait past idle_timeout without touching it.
sleep <idle_timeout + tick>
clr sessions                 # no hosted sessions, daemon still up

# Wait past linger with nothing hosted.
sleep <linger + tick>
clr daemon status ; echo "exit=$?"      # not running, exit 1

# The conversation was released, not destroyed.
clr chat "what did I say first?"        # resumes; same id as before
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/serve.rs` | `Daemon::reap`, `should_exit`, the refresh inside it |
| source | `src/table.rs` | `HostedSession::last_active`, `shutdown` grace |
| doc | [009_session_resume.md](009_session_resume.md) | Prerequisite — what makes a release recoverable |
| doc | [008_turn_state.md](008_turn_state.md) | The `busy` flag idle depends on, and the no-clock note this extends |
| doc | [001_single_instance.md](001_single_instance.md) | The lock ordering an exiting daemon must honour |
| doc | [003_session_table.md](003_session_table.md) | The table reaped from |
| test | `tests/serve_test.rs` | Dispatch against a real socket and real children |
