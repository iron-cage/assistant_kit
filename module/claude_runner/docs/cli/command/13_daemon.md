# CLI Command: daemon

### Description

Manage the single session daemon — the long-lived process that owns interactive
`claude` sessions on terminals of their own, so a session outlives the command that
started it. `clr daemon status` answers whether one is running, `start` and `stop`
move it between those two states, and `log` prints the path of the file it writes to.

-- **Parameters:** `[SUBCOMMAND]` (`status` | `start` | `stop` | `log` | `help`)
-- **Exit Codes:** 0 | 1 — meaning differs per subcommand, see Exit Codes below
-- **Forms:** query (`status`, `log`) | transition (`start`, `stop`)

### Syntax

```sh
clr daemon [status]
clr daemon start
clr daemon stop
clr daemon log
clr daemon help
```

### Parameters

| # | Name | Required | Purpose |
|---|------|----------|---------|
| 1 | `[SUBCOMMAND]` | No — defaults to `status` | Which of the five operations to perform |

Bare `clr daemon` is `clr daemon status`: the question asked most often, and the only
one of the five with no side effect.

### Exit Codes

Each subcommand's exit code answers a different question, so the codes are not
interchangeable:

| Subcommand | 0 | 1 |
|------------|---|---|
| `status` | A daemon is running | Nothing is running |
| `start` | A daemon is running when the command returns — whether this call started it or found it | It could not be started |
| `stop` | Nothing is running when the command returns — whether this call stopped it or found nothing | It acknowledged the stop and kept answering |
| `log` | The path was printed | `HOME` is not set, so no path exists to print |

`start` and `stop` describe the state on return, not what this particular call did.
That is what makes `clr daemon status || clr daemon start` work, and what makes both
safe to run twice.

**Algorithm — `status` (3 steps):**
1. Send `ping` over the socket, with a 2-second timeout.
2. No answer → report `not running`, print the socket path, exit 1.
3. An answer → report the version and log path, then send `list_sessions` and print one line per hosted session; exit 0.

**Algorithm — `start` (4 steps):**
1. Probe first. Already answering → print the version and exit 0 without spawning anything.
2. Spawn `<current-exe> __daemon_serve`, detached: its own process group, stdin from `/dev/null`, stdout and stderr appended to the log.
3. Probe every 50ms for up to 10 seconds. Answering → print the version and socket path, exit 0.
4. The child exited, or the timeout elapsed → print the last 20 log lines and exit 1. A child that exited is re-probed once first, because losing a start race to another `clr daemon start` looks exactly the same from here.

**Algorithm — `stop` (4 steps):**
1. Probe. Nothing there → print `daemon not running`, exit 0.
2. Send `stop_daemon` and read the acknowledgement.
3. Probe every 50ms for up to 10 seconds until nothing answers → print `daemon stopped`, exit 0.
4. Still answering after 10 seconds → report that, print the log path, exit 1.

### Examples

```sh
# Is one running, and what is it hosting?
clr daemon status

# Start it if it is not already running
clr daemon status || clr daemon start

# Watch what it does
tail -f "$( clr daemon log )"

# Read one of its timestamps back
date -d @1787764397

# Shut every session down and stop it
clr daemon stop
```

### Notes

**The daemon is this same binary.** `start` spawns `std::env::current_exe()` with the
hidden `__daemon_serve` token, exactly as `clr query` spawns `__query_daemon`. A
separate `claude_daemon` executable would have to be *found* — by `PATH`, or by
guessing at a sibling of the current one — and an older copy found that way speaks an
older protocol to a newer client. `current_exe()` cannot be the wrong version of
itself.

**Detachment needs no `setsid`.** Three things make the daemon outlive the shell that
started it, none of them an FFI call: it runs in its own process group
(`CommandExt::process_group( 0 )`), so the terminal's `SIGINT`/`SIGQUIT`/`SIGTSTP` —
sent to the *foreground* group — never reach it; `clr daemon start` exits immediately,
so it is reparented to init and is no longer a job the shell will `SIGHUP`; and its
stdio never points at a terminal. Unsafe stays confined to `claude_pty_core`, the crate
that owns it.

**Stopping is a request, not a signal.** `SIGTERM` tells the sender nothing — not
whether it arrived, not whether the sessions came down cleanly, not whether there was a
daemon at all. `stop` sends `stop_daemon` over the socket and gets an answer on the
same connection. The daemon sets a flag inside the request and tears its sessions down
*after* replying, so a slow session cannot make the daemon look unresponsive; `stop`
then waits for the socket to actually go quiet, because acknowledged is not stopped.

**At most one daemon, enforced by `flock`.** `__daemon_serve` takes an exclusive
advisory lock on `<claude-home>/-daemon/instance.lock` before binding anything. A
second one that loses the race exits 0 quietly — whoever started it wanted a daemon
running, and there is one. See
[claude_daemon_core's single-instance feature](../../../../claude_daemon_core/docs/feature/001_single_instance.md)
for why this is a lock and not a PID file.

**Files.** All under `<claude-home>/-daemon/`, hyphen-prefixed so the workspace's
global `-*` ignore rule keeps machine-local state out of version control:
`instance.lock`, `daemon.sock`, `daemon.log`. `stop` removes the socket and leaves the
log — a daemon that keeps dying at startup is only debuggable if stopping does not
erase what it wrote.

**Log timestamps are epoch seconds.** No dependency, sorts correctly, and
`date -d @<n>` reads one back.

**Error messages:**
- `Error: unknown daemon subcommand "<token>"` — followed by a pointer to `clr daemon help`.
- `Error: cannot resolve the Claude home — HOME is not set` — `DaemonPaths::new()` returned `None`.
- `Error: cannot start the daemon: <reason>` — the spawn itself failed.
- `Error: the daemon exited during startup` / `Error: the daemon did not answer in time` — both followed by the last 20 lines of the log.
- `Error: the daemon refused to stop: <reason>` — the `stop_daemon` request failed.
- `Error: the daemon acknowledged the stop but is still answering` — followed by the log path.

### Referenced Command Group

Evaluated against `query` under the strict [command_group](../command_group/readme.md)
identity test (same dispatch function, same parameter set) — does not qualify.
`dispatch_daemon()` (`src/cli/daemon.rs`) and `dispatch_query()` (`src/cli/query.rs`)
share a *technique* — `current_exe()` plus a hidden `__`-prefixed token — and nothing
else: no cross-calls, no shared parameters, and opposite cardinality (one daemon for
every session, versus one `__query_daemon` per session). Technique reuse is not a
command group.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`query`](10_query.md) | The same detached-child technique, per-session instead of one process for all of them |
| 2 | [`ps`](06_ps.md) | Lists `claude` processes found by scanning `/proc`; `daemon status` lists only the sessions this daemon hosts |

### Referenced Parameter Groups

None. `daemon` takes a subcommand and no flags.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 32 | [032_hosted_session_chat.md](../user_story/032_hosted_session_chat.md) | Developer |

The daemon has no user story of its own: it exists to carry interactive sessions for
`chat`, and every acceptance criterion that touches it — AC-8 (idempotent lifecycle),
AC-9 (one daemon), AC-10 (outlives the shell), AC-14 (teardown) — is written from the
point of view of someone chatting, not of someone managing a process.

---

**Category:** Session management
**Complexity:** 8
**API Requirement:** Write
**Idempotent:** Yes — `start` and `stop` both describe the state on return, not the transition
**Risk Level:** Medium
