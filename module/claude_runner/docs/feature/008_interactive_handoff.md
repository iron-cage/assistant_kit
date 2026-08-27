# Feature: Interactive Handoff

### Scope

- **Purpose**: Let one conversation be reached both by the print-mode commands and by an interactive session, without two processes ever holding it at once — by releasing the daemon's session before opening it on the caller's terminal.
- **In Scope**: The probe an interactive `clr` performs before spawning, the release request it sends, the resumed interactive spawn, and what happens when the hosted session is mid-turn.
- **Out of Scope**: Re-opening a released conversation (→ `claude_daemon_core/docs/feature/009_session_resume.md`), releasing a session because nobody used it (→ `claude_daemon_core/docs/feature/010_session_reaping.md`), the daemon's own lifecycle (→ `docs/cli/command/13_daemon.md`).

### Why This Exists

`clr chat` and interactive `clr` can currently target the same conversation at the same time,
and nothing stops them.

The daemon hosts session `S` in directory `D`, holding `claude` on a pseudo-terminal and
appending to `<claude home>/projects/<encoded D>/S.jsonl`. The user then runs `clr` in `D`.
`clr` spawns `claude -c`, which continues *the most recent conversation in that directory* —
which is `S`. Two live processes now believe they own one conversation and one transcript.

There is no error for this. There is not even a warning: both processes work, both write, and
what the transcript ends up containing is whichever interleaving the filesystem produced.

The narrower framing — "you get a duplicate session" — undersells it. The real loss is that
there is no way to hold one conversation and reach it *both* ways: type into it directly when
that is what you want, script against it with `clr chat` when that is what you want. That
combination is the reason the daemon exists, and today it is exactly the thing that breaks.

### The Handoff

Before an interactive spawn, in `dispatch_run`:

1. **Probe — never start.** `probe()`, not `ensure_running()`. An interactive `clr` that
   started a daemon in order to ask it a question would be doing the thing
   `docs/cli/command/15_sessions.md` already refuses to do: changing the state it is asking
   about. No daemon means nothing is hosted, which is a complete answer, and the spawn proceeds
   as it does today.
2. **Match** hosted sessions on canonicalised working directory — the same rule `clr chat`
   resolves by, so both commands agree on what "this directory's session" means.
3. **Busy → stop.** See below.
4. **Idle → release.** `Request::Shutdown { session_id }`, which already exists. It closes the
   descriptors, lets `claude` exit through its own shutdown code and flush the transcript, and
   waits out a bounded grace period.
5. **Spawn interactively, resumed** — `claude --resume <session_id>` on the caller's terminal,
   rather than the `-c` that made the collision possible.

The daemon is not stopped, asked to stop, or signalled. It loses one session and keeps
serving; if that was its last, its own linger clock starts, which is
`claude_daemon_core/docs/feature/010_session_reaping.md`'s business and not this feature's.

### A Busy Session Is Not Taken

Releasing a session mid-turn discards an answer that is being generated and cannot be asked
for again for free. So a busy match stops the command rather than resolving it:

```
Error: the session in this directory is mid-turn (4f2c8a1e-…).
       Wait for it to finish, or watch it with `clr sessions`.
```

Naming the remedy rather than only the problem, in the manner
[005_session_registration.md](../../../claude_daemon_core/docs/feature/005_session_registration.md)
already sets for a failure whose cause is invisible from here.

**Refusing rather than waiting is settled, not provisional.** The two failure modes are not
symmetric. Refusing wrongly costs a retry the caller can see and act on. Waiting wrongly costs
an indefinite hang on a foreground command, plus interrupt semantics that would have to be
invented and trusted — and a session mid-way through a forty-minute autonomous turn is
indistinguishable from one four seconds from done, so the daemon cannot bound the wait
honestly.

A `--wait` flag remains additive if it is ever wanted. Changing the default from wait to
refuse afterwards would not be, because callers build habits around a default that blocks.

### The Return Trip Is Free

Nothing needs to hand the session back. The user quits the interactive session; the next
`clr chat` in that directory finds nothing hosted, spawns, and — because the daemon remembers
which conversation last occupied that directory — resumes rather than starting over. One
conversation, reachable both ways, with the handoff in each direction costing one process
restart and no lost history.

This is why the prerequisite is strict. Without resume, this feature converts a *collision*
into a *deletion*: the interactive session takes the conversation and `clr chat` can never get
it back.

### What Is Not Yet Settled

- **Which commands participate.** `clr` and `clr run` clearly. Whether `clr ask` and `clr topic`
  do is **TBD**: they are short-lived print-mode invocations, so the collision window is smaller,
  but it is not zero and the argument for excluding them is convenience rather than safety.
- **The unhosted direction.** The daemon cannot see interactive sessions it does not host, so
  `clr chat` cannot detect the reverse collision the same way. `clr ps` can see them. Whether
  `clr chat` should warn when an unhosted `claude` is already running in the target directory
  is **TBD**.

### Verification

```bash
cargo test -p claude_runner --test chat_command_test
```

Against a live daemon — the collision this feature removes:

```bash
cd "$( mktemp -d )"
clr chat "remember the word pineapple"
ID=$( clr sessions --json | jq -r '.[0].session_id' )

# Interactive clr in the same directory releases it first.
clr            # ask it the word, then quit
clr sessions   # empty during the interactive session — the daemon released it

# And the conversation comes back to chat afterwards, same id.
clr chat "what word did I ask you to remember?"
clr sessions --json | jq -r '.[0].session_id'
echo "was: $ID"
```

The failure this pins is not a crash. Before the handoff, both processes ran and both
answered; the damage was in the transcript.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/cli/run.rs` | The probe and release, before the interactive spawn |
| source | `src/cli/daemon.rs` | `probe`, and the spawner that emits `--resume` |
| doc | `claude_daemon_core/docs/feature/009_session_resume.md` | Prerequisite — what makes the return trip possible |
| doc | `claude_daemon_core/docs/feature/010_session_reaping.md` | The other release path, and the linger clock this can start |
| doc | [../cli/command/14_chat.md](../cli/command/14_chat.md) | The cwd-matching rule this reuses |
| doc | [../cli/command/15_sessions.md](../cli/command/15_sessions.md) | The probe-don't-start precedent |
| doc | [../cli/user_story/032_hosted_session_chat.md](../cli/user_story/032_hosted_session_chat.md) | Acceptance criteria for the hosted-session stack |
| test | `tests/docs/cli/command/14_chat.md` | Test-case planning for the chat surface |
