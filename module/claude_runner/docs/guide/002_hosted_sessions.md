# Guide: Hosted Sessions

Hold one real interactive Claude Code conversation open across many separate shell commands — each command reading like print mode (prompt in, answer out, shell prompt back), while the session it talked to stays alive between them.

This is the print-mode *shape* over a real interactive session. `clr ask` spawns a
subprocess, prints, and the subprocess dies with the conversation in it. `clr chat`
sends a prompt to a session running on a terminal a daemon owns, so the next `clr chat`
continues rather than starts over.

## Prerequisites

Each fact below was verified against the cited source before this guide was written.

| # | Prerequisite Fact | Verification Source |
|---|-------------------|---------------------|
| 1 | `chat`, `sessions`, and `daemon` exist as dispatchable subcommands | `src/cli/mod.rs` — `KNOWN_SUBCOMMANDS`; `src/lib.rs` — `dispatch_chat` / `dispatch_sessions` / `dispatch_daemon` |
| 2 | The daemon's state is `<claude-home>/-daemon/{instance.lock,daemon.sock,daemon.log}` | `claude_daemon_core/src/paths.rs` — `RUNTIME_DIR_NAME = "-daemon"`, `LOCK_FILE_NAME`, `SOCKET_FILE_NAME`, `LOG_FILE_NAME` |
| 3 | `<claude-home>` comes from `HOME` and nothing else; unset `HOME` → no paths at all | `claude_daemon_core/src/paths.rs` — `DaemonPaths::new()` delegates to `ClaudePaths::new()`, which returns `None` without `HOME` |
| 4 | `chat` starts a daemon if none is running; `sessions` never does | `src/cli/chat.rs` — `ensure_running( &paths )` before anything else; `src/cli/sessions.rs:52` — `probe( &socket )` only, `ensure_running` never imported |
| 5 | With no `--session`, the session is chosen by **canonicalised** working directory | `src/cli/chat.rs:267` — `args.dir.canonicalize()`, compared against each session's own canonicalised `cwd` |
| 6 | No directory match → a session is spawned there, then given 3s to draw before the prompt is sent | `src/cli/chat.rs` — `spawn_session()`, then `BANNER_SETTLE : Duration = Duration::from_secs( 3 )` |
| 7 | The turn ends only when the session is **both** idle and quiet for 8 polls (~0.8s) | `src/cli/chat.rs` — `QUIET_POLLS : usize = 8`, `POLL = 100ms`, combined with the daemon's busy flag |
| 8 | The printed answer comes from the session transcript, not the terminal; `--raw` prints the terminal | `src/cli/chat_answer.rs`; `src/cli/chat.rs` module docs — "Knowing *when* is not knowing *what*" |
| 9 | At most one daemon, enforced by an exclusive `flock` on `instance.lock` | `claude_daemon_core/docs/feature/001_single_instance.md` |
| 10 | The daemon survives the shell: own process group, no controlling terminal, reparented to init | `src/cli/daemon.rs:317` — `command.process_group( 0 )`; `SERVE_TOKEN = "__daemon_serve"` |
| 11 | Sessions are started with background-task reporting on, which is what makes `idle` trustworthy | `src/cli/daemon.rs:462` — `.env( BG_TASKS_REPORT_RUNNING_ENV, "1" )` |
| 12 | A `claude` parked on a first-run prompt never registers, and `chat` says so by name | `src/cli/chat.rs` — the `never registered a conversation id` branch and its three-line hint |

**Two environment prerequisites**, neither optional, both learned by hitting them:

```sh
# 1. A `claude` past its first run. A session parked on a theme picker or a trust
#    dialog never opens a conversation, so it never registers, so the spawn fails
#    with no visible cause. Answer the prompts once, here, in this environment.
claude          # then quit it

# 2. Confirm the state that run wrote. Both must be true for the directory you
#    intend to chat in.
grep -o '"hasCompletedOnboarding":[^,]*' "$HOME/.claude.json"
grep -o '"hasTrustDialogAccepted":[^,]*' "$HOME/.claude.json" | head -1
```

Everything the stack touches hangs off `HOME` — runtime dir, lock, socket, the registry
it scans, the transcripts it reads answers from. To keep this out of your real one, set
`HOME` to a directory that satisfies both checks above; see Open Decisions.

**Placeholder Values used below** — resolve each fresh, they are not fixed constants:

| Placeholder | Meaning | Discovery command |
|-------------|---------|-------------------|
| `<session>` | A conversation id the daemon hosts | `clr sessions` (the `SESSION` column), or `clr sessions --json \| jq -r '.[0].session_id'` |
| `<work>` | A directory to hold the first session | `W=$( mktemp -d ) && cd "$W" && pwd` |
| `<other>` | A second directory, to prove sessions are per-directory | `O=$( mktemp -d ) && echo "$O"` |

## Phase 1 — Confirm the command surface

Read-only; no State-Check Sandwich needed.

```sh
# The three commands and how they divide the work
clr chat help
clr sessions help
clr daemon help

# Where this daemon would keep its state, before one exists
clr daemon log
dirname "$( clr daemon log )"
```

## Phase 2 — The first chat, in a fresh directory

State-changing: starts a daemon, starts a session, and consumes a turn.

```sh
# 1. HELP
clr chat help

# 2. BEFORE — nothing running, nothing hosted, no socket on disk
clr daemon status ; echo "daemon exit=$?"     # expect: not running, exit 1
clr sessions      ; echo "sessions exit=$?"   # expect: nothing hosted, exit 0
ls "$( dirname "$( clr daemon log )" )/daemon.sock" 2>&1

# 3. ACTION
W=$( mktemp -d ) && cd "$W"
clr chat "Reply with exactly one word and nothing else: pineapple" | cat -A

# 4. AFTER — identical to step 2's commands
clr daemon status ; echo "daemon exit=$?"     # expect: running, exit 0
clr sessions      ; echo "sessions exit=$?"   # expect: one row, exit 0
ls "$( dirname "$( clr daemon log )" )/daemon.sock"
```

`cat -A` is the point of step 3, not decoration. Expect exactly `pineapple$` — one word,
one newline, and no box rules, no `❯` prompt line, no status bar, no spinner frames. The
`Starting a session in … ` notice goes to stderr, so the pipe shows the answer alone.

The before/after pair is the same two commands either side of the mutation, so the delta
— nothing hosted, then one session hosted — is read off a command distinct from the one
that performed it (GD003).

## Phase 3 — The same directory continues the conversation

State-changing: consumes a turn. The state check is that the session count does **not**
change, which is exactly the claim being tested.

```sh
# 1. HELP
clr sessions help

# 2. BEFORE — one session, and note its id
clr sessions

# 3. ACTION — no --session, no --dir; the cwd resolves it
clr chat "What single word did I ask you to reply with a moment ago? Answer with just that word."

# 4. AFTER — still one session, still the same id
clr sessions
```

Answering `pineapple` is only possible if this reached the same session rather than a
fresh one. A second row appearing here means directory resolution failed and each call
is starting over — the difference between this and `clr ask`, and the reason the daemon
exists.

## Phase 4 — A second directory gets its own session

```sh
# BEFORE
clr sessions          # one row

# ACTION — --dir names which directory's session to talk to, and spawns there if none
O=$( mktemp -d )
clr chat "Reply with exactly one word: tangerine" --dir "$O"

# AFTER — two rows, two ids, two CWDs
clr sessions
```

`--dir` is a lookup key first and a spawn target second, so it is not checked for
existence up front; a bad path fails at the spawn, which reports it. Contrast every other
command's `--dir`, which validates before spawning ([`008_dir.md`](../cli/param/008_dir.md)).

## Phase 5 — Address a session by id, from anywhere

```sh
# BEFORE — the id is the handle, and `sessions` is where it comes from
clr sessions
SESSION=$( clr sessions --json | jq -r '.[] | select( .cwd == "'"$O"'" ) | .session_id' )
echo "$SESSION"

# ACTION — run this from a third directory entirely; --session outranks the cwd
cd /
clr chat "Which fruit did I just name? One word." --session "$SESSION"

# AFTER — still two sessions; addressing one did not create a third
clr sessions
```

`--session` is honoured even when the session is busy and even when it lives somewhere
else — the caller named it. The conversation id, not the PID, is the durable handle:
Claude Code re-hosts a session with `--fork-session` on auto-update or recovery, and the
PID changes while the id does not.

## Phase 6 — The terminal underneath, and the answer on top

```sh
# The default: the answer the transcript recorded
clr chat "Say OK." --dir "$W"

# The same turn's terminal bytes — escape sequences, box rules, input box
clr chat "Say OK again." --dir "$W" --raw | head -20
```

This contrast is what makes the default meaningful. The chrome is still there and still
reachable; the default simply stops printing it, because filtering a TUI layout `clr`
does not own would pin a release to someone else's interface.

## Phase 7 — Long prompts, and why they are worth a check

```sh
for n in 26 54 68 79 88 137; do
  msg=$( printf 'Reply with one word: ok%*s' "$(( n - 22 ))" '' | tr ' ' 'x' )
  printf '%3s bytes: ' "$n"
  timeout 120 clr chat "$msg" --dir "$W" 2>/dev/null | head -1
done
```

Every length must answer. This pins a real regression: with the prompt's text and its
submitting carriage return written back to back, prompts up to roughly 55 bytes submitted
and everything longer silently did not — the text landed in the input box and stayed
there. No error on either side. `send` now pauses 200ms between the two writes so the
return cannot be read as part of a paste.

## Phase 8 — Teardown

State-changing: stops every session and the daemon with them.

```sh
# 1. HELP
clr daemon help

# 2. BEFORE
clr daemon status ; echo "exit=$?"            # expect: running, exit 0
clr sessions

# 3. ACTION
clr daemon stop

# 4. AFTER — identical to step 2's commands
clr daemon status ; echo "exit=$?"            # expect: not running, exit 1
clr sessions      ; echo "exit=$?"            # expect: nothing hosted, exit 0
ls "$( dirname "$( clr daemon log )" )/daemon.sock" 2>&1   # gone
```

`stop` removes the socket and leaves the log, because a daemon that keeps dying at
startup is only debuggable if stopping does not erase what it wrote. `start` and `stop`
both describe the state on *return*, not what the call did — which is what makes
`clr daemon status || clr daemon start` work and both safe to run twice.

## Verification

The end goal — a conversation that survives the command, addressable from any directory,
and torn down on demand — is confirmed when all five hold:

```sh
# 1. The answer is a message, not a rendered terminal
clr chat "Reply with exactly one word and nothing else: pineapple" | cat -A
#    -> exactly `pineapple$`

# 2. The session outlived the command that made it
clr sessions
#    -> one row, holding the directory from step 1

# 3. It remembers, so the second command reached the same conversation
clr chat "What word did I ask for? Just the word."
#    -> pineapple, and `clr sessions` still shows ONE row, not two

# 4. It outlives the shell, not just the command
clr daemon status && echo "still up in a brand new shell"

# 5. Asking what is hosted starts nothing
clr daemon stop
clr sessions ; echo "exit=$?"
ls "$( dirname "$( clr daemon log )" )/daemon.sock" 2>&1
#    -> "no daemon running" on stderr, exit 0, and NO socket created by asking
```

Point 3 is the one that actually proves hosting worked. A second row appearing there
means each `clr chat` started its own session and the conversation is not being carried —
the command still printed an answer, so nothing looks wrong until you notice it has
forgotten everything.

Point 5 is the asymmetry worth internalising: `chat` auto-starts a daemon because a
caller asking to talk to a session wants one, and `sessions` does not, because a question
that starts a process to answer itself has changed the thing it was asking about.

## Open Decisions

- **Whether to isolate `HOME`.** Everything here lands in `$HOME/.claude/` — the runtime
  dir, the lock, the socket, the log, and the transcripts answers are read from. Pointing
  `HOME` at a scratch directory keeps a real one clean and still exercises the real
  default-path code with no test-only override, but that scratch `HOME` needs its own
  completed first run (see Prerequisites) or every spawn fails. This guide does not choose,
  because the answer depends on whether you are trying the stack out or using it.
- **When sessions end.** Nothing reaps an idle session. A hosted session lives until
  `clr daemon stop` takes the daemon and its sessions down together, or the machine
  reboots — each one holding a `claude` process and a pty for as long as it exists.
  Whether that is left running between work sessions is a per-user call; there is no
  per-session stop, only the daemon-wide one.
- **The 300-second default timeout.** Long enough for ordinary turns, short enough to
  give up on a wedged one. A turn cut short by it still exits 0 with a warning, and
  `clr chat` again prints the rest — so raising it with `--timeout` is a convenience,
  not a correctness fix. Whether a refactor-sized prompt is better served by a large
  `--timeout` or by two chats is left to the caller.

## Related

| Type | Path | Relationship |
|------|------|--------------|
| command | [`../cli/command/14_chat.md`](../cli/command/14_chat.md) | `chat` specification — parameters, the three algorithms, exit codes |
| command | [`../cli/command/15_sessions.md`](../cli/command/15_sessions.md) | `sessions` specification — listing, `--json`, why it starts nothing |
| command | [`../cli/command/13_daemon.md`](../cli/command/13_daemon.md) | `daemon` specification — status/start/stop/log and detachment |
| command | [`../cli/command/05_ask.md`](../cli/command/05_ask.md) | `ask` — the same shape in print mode, with no session surviving it |
| command | [`../cli/command/06_ps.md`](../cli/command/06_ps.md) | `ps` — every Claude Code process on the machine, hosted or not |
| param | [`../cli/param/008_dir.md`](../cli/param/008_dir.md) | `--dir` — a lookup key here, a validated spawn target everywhere else |
| user story | [`../cli/user_story/032_hosted_session_chat.md`](../cli/user_story/032_hosted_session_chat.md) | Acceptance criteria AC-1 … AC-14 for the whole stack |
| feature | [`../../../claude_daemon_core/docs/feature/001_single_instance.md`](../../../claude_daemon_core/docs/feature/001_single_instance.md) | Why one daemon is a `flock` and not a PID file |
| feature | [`../../../claude_daemon_core/docs/feature/006_serving_clients.md`](../../../claude_daemon_core/docs/feature/006_serving_clients.md) | The 200ms submit pause behind Phase 7 |
| test plan | [`../../tests/manual/readme.md`](../../tests/manual/readme.md) | MD-1 … MD-6 — the manual cases these phases are drawn from |
