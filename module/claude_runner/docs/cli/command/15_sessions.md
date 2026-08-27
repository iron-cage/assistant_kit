# CLI Command: sessions

### Description

List the interactive sessions the daemon is currently hosting — one row each, with the
conversation id `clr chat --session` takes, the process id, whether a turn is in flight,
and the directory it runs in.

Not the same question as `clr ps`, which scans `/proc` and finds every Claude Code
process on the machine however it was started. This lists only what *this daemon owns*:
the sessions `clr chat` can address and `clr daemon stop` would take down. A session that
appears in `clr ps` and not here is one the daemon cannot talk to.

-- **Parameters:** `--json`
-- **Exit Codes:** 0 (the list was printed, empty or not) | 1 (the daemon would not answer)
-- **Forms:** query — no side effects, and deliberately no daemon started

### Syntax

```sh
clr sessions
clr sessions --json
clr sessions help
```

### Parameters

| # | Name | Required | Default | Purpose |
|---|------|----------|---------|---------|
| 1 | `--json` | No | off | Print the daemon's own list verbatim instead of a table |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | The list was printed — including when it is empty, and including when no daemon is running |
| 1 | An unknown option, or a daemon that answered `ping` and then would not list |

No daemon is not a failure. There is nothing hosted, which is a complete and correct
answer to the question asked. It is reported on stderr so it cannot be mistaken for a row
of output, and `clr sessions --json` still prints `[]` so a consumer piping into a parser
does not have to special-case it.

**Algorithm (6 steps):**
1. Parse flags. An unknown option exits 1 before anything is contacted.
2. Probe the socket. Nothing answering → explain on stderr, print `[]` if `--json`, exit 0.
3. Send `list_sessions`. The daemon refreshes each session's busy flag from Claude Code's own registry while answering this.
4. `--json` → pretty-print the daemon's array and exit 0.
5. Empty → `No hosted sessions.` on stdout, exit 0.
6. Otherwise render the table and exit 0.

### Examples

```sh
# What is hosted?
clr sessions

# For a script
clr sessions --json | jq -r '.[] | select(.busy) | .session_id'

# Everything Claude Code on this machine, hosted or not
clr ps

# Talk to one of them
clr chat "carry on" --session "$( clr sessions --json | jq -r '.[0].session_id' )"
```

### Output

```
Hosted Sessions · 2 total · 1 busy

#  SESSION                               PID     STATE  CWD
1  4f2c8a1e-...                          182347  busy   ~/work/parser
2  9b71d05c-...                          182502  idle   ~/work/other-project
```

The `SESSION` column is never abbreviated. It is the handle `clr chat --session` takes,
and a handle you have to retype from memory is not a handle. `CWD` is shortened the way
`clr ps` shortens it.

### Notes

**This does not start a daemon, and `clr chat` does.** The asymmetry is deliberate. A
client asking to talk to a session wants a session, and the daemon is how it gets one.
Asking what is hosted is a question, and a question that starts a process to answer
itself has changed the thing it was asking about — the honest answer to "what is hosted"
when nothing is running is "nothing", not "one empty daemon, which I just made for you".

**`STATE` comes from Claude Code, not from the daemon's guesswork.** Each session writes
its own status into the registry, and the daemon reads it back through a turn watcher —
edge-triggered, so a session observed already idle is not mistaken for one that just
finished. The refresh happens while answering this request rather than on a timer,
because the daemon is single-threaded and spends its life blocked in `accept`: between
requests there is nobody to sample and nobody to sample for.

**`PID` is advisory.** Claude Code re-hosts a session with `--fork-session` on auto-update
or recovery, and the new process has a different PID. The conversation id is what
survives that, which is why it and not the PID is the handle everything else takes.

**Error messages:**
- `Error: unknown option "<token>" for 'clr sessions'` — followed by a pointer to help.
- `Error: the daemon would not list its sessions: <reason>` — it answered `ping` and then failed.
- `Error: the daemon's session list did not parse: <reason>` — a protocol mismatch between client and daemon.

**Informational (stderr, exit 0):**
- `No session daemon is running — nothing is hosted.` — followed by how to start one.

### Referenced Command Group

Evaluated against `ps` under the strict [command_group](../command_group/readme.md)
identity test (same dispatch function, same parameter set) — does not qualify. They
answer adjacent questions and share a table renderer, but `dispatch_sessions()`
(`src/cli/sessions.rs`) asks a daemon over a socket while `dispatch_ps()`
(`src/cli/ps.rs`) scans `/proc`, and they share no parameters. Presentation reuse is not
a command group.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`chat`](14_chat.md) | Talks to the sessions listed here; takes the `SESSION` column as `--session` |
| 2 | [`daemon`](13_daemon.md) | `daemon status` gives a shorter version of this alongside the daemon's own state |
| 3 | [`ps`](06_ps.md) | Every Claude Code process on the machine, hosted or not |

### Referenced Parameter Groups

None. `sessions` takes one flag of its own and forwards nothing.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 32 | [032_hosted_session_chat.md](../user_story/032_hosted_session_chat.md) | Developer |

AC-4 (the session survives) and AC-7 (asking what is hosted starts nothing) are this
command's half of that story.

---

**Category:** Session management
**Complexity:** 4
**API Requirement:** Read
**Idempotent:** Yes — a pure query with no side effects, including on the daemon's existence
**Risk Level:** Low
