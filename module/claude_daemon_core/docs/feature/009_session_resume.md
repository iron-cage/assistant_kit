# Feature: Session Resume

### Scope

- **Purpose**: Start a hosted session attached to a conversation that already exists, rather than a fresh one — so releasing a session from the daemon stops meaning destroying the conversation inside it.
- **In Scope**: The spawner's resume parameter, resolving which conversation last occupied a directory, and the conditions under which a spawn resumes instead of starting over.
- **Out of Scope**: When a session is released (→ [010_session_reaping.md](010_session_reaping.md), and `claude_runner/docs/feature/008_interactive_handoff.md`), the flag a concrete spawner emits (→ `claude_runner/docs/cli/command/14_chat.md`), learning the id of a session just started (→ [005_session_registration.md](005_session_registration.md)).

### Why This Exists

Every feature that ends a hosted session needs this one first.

The daemon's whole promise is that a conversation survives the command that spoke to it.
Two planned features deliberately end sessions — reaping an idle one, and handing one to an
interactive client — and without resume both of them break that promise in the worst
available way. Trace it:

1. The daemon releases session `S`, hosted in directory `D`. The transcript survives on disk.
2. The next `clr chat` in `D` asks the daemon for its sessions and matches on cwd.
3. Nothing matches, so it spawns — and gets a **new** conversation with a **new** id.
4. An answer prints. Nothing errors. The conversation is gone.

The failure is silent, and it presents as a model that has forgotten everything rather than
as a fault. A release path without a resume path is not a smaller feature; it is a
correctness hole in the feature that already shipped.

### What the Underlying Tool Provides

Verified against `claude --help`:

```
-r, --resume [value]    Resume a conversation by session ID, or
                        open interactive picker with optional search term
--fork-session          When resuming, create a new session ID
                        instead of reusing the original (use with --resume or --continue)
```

Two consequences, and both are load-bearing:

**Resume reuses the original id.** `--fork-session` exists precisely to opt *out* of that,
which means the default is the behaviour this feature needs: a resumed session answers to
the same conversation id it had before. Every `--session <ID>` handle a user wrote down
stays valid across a release/resume cycle, and the daemon's table can be re-keyed by the id
it already knows.

**A bare `--resume` is a trap.** With no value it opens an interactive picker. A session
spawned that way parks on the picker, never opens a conversation, never registers, and dies
at the registration timeout with no visible cause — the identical failure mode already
documented for first-run prompts in [005_session_registration.md](005_session_registration.md).
A resuming spawner must therefore treat "resume with no id" as a programming error rather
than as a default, and never construct the flag without a value beside it.

### Where the Decision Lives

Split, because the two halves are different kinds of knowledge.

This crate owns **whether** to resume: when a spawn finds no live session for a directory, it
asks which conversation last occupied that directory, and attaches to it.

**That question is answered from disk, not from memory.** An in-memory map would be the
obvious implementation and it is wrong, for a reason that only became visible once
[010_session_reaping.md](010_session_reaping.md) settled that the daemon exits when idle:

> session reaped at 30 min → daemon exits at 35 min → **the map dies with it** → the next
> `clr chat` in that directory has nothing to resume and silently starts over.

That is the exact failure this feature exists to prevent, reintroduced by the feature that
depends on it, in precisely the window reaping creates. A persisted map would fix it and cost
a file format, a write path, and a staleness story.

The transcript directory already *is* the map. `claude_storage_core` reads it, this crate
already depends on that, and every conversation in a directory is on disk with a timestamp.
Deriving "the id that last occupied this cwd" from there is precise — an actual id, not
`--continue`'s implicit choice — survives daemon restarts for free, and has nothing to evict.

The caller owns **how**. `Daemon` is generic over its spawner precisely so this crate never
learns what program a session runs, and that stays true: the spawner signature gains a
resume parameter, and translating `Some( id )` into a command-line flag belongs to whoever
wrote the spawner.

```rust,ignore
// The spawner signature grows one parameter.
//
// `None`  — start a new conversation in `cwd`.
// `Some`  — attach to this conversation, which already exists.
S : FnMut( &Path, Option< &str > ) -> Result< PtySession >
```

The concrete translation, in `claude_runner`:

```rust,ignore
fn spawn_claude( cwd : &Path, resume : Option< &str > ) -> Result< PtySession >
{
  let mut config = SessionConfig::new( "claude" )
    .cwd( cwd )
    .env( BG_TASKS_REPORT_RUNNING_ENV, "1" );

  // Never a bare `--resume`: with no value it opens a picker, and a session
  // parked on a picker never registers.
  if let Some( session_id ) = resume
  {
    config = config.arg( "--resume" ).arg( session_id );
  }

  PtySession::spawn( &config ).map_err( Error::Pty )
}
```

### What Is Not Yet Settled

- **Behaviour on a terminal — settled.** Observed against a real `claude` (HS-0): `--resume
  <id>` brings up a usable interactive session immediately, with no picker and no keystroke
  needed first. Everything in this document assuming that turned out correct, and nothing
  about `Daemon::spawn` had to special-case the other outcome.
- **Which transcript wins when a directory holds several — settled.** Most-recently-modified,
  unconditionally: [`src/serve.rs`](../../src/serve.rs)'s `spawn` calls
  `claude_storage_core::most_recent_session_id(cwd)`, the same primitive continuation
  detection already uses, rather than tracking "the most recent this daemon previously
  hosted." That rejected alternative would need daemon-side memory — exactly the thing
  [Where the Decision Lives](#where-the-decision-lives) already ruled out for the resume
  question generally, for the same reaping-window reason.
- **Whether a resumed session re-registers, and how fast — still open.** [005_session_registration.md](005_session_registration.md)'s
  wait is unchanged: `Daemon::spawn` polls for the spawned pid exactly as it does for a fresh
  conversation, on the same `registration_timeout`, relying on nothing more than "resume
  reuses the original id" (verified against `claude --help`, load-bearing above). Whether a
  *resumed* Claude Code process republishes a registry record at all, and within the normal
  window, is a fact about the real tool this crate cannot observe from a test fixture —
  `tests/serve_test.rs`'s `srv16`/`srv17` cover the daemon's own logic against a fixture
  spawner that always re-registers promptly, which only proves the daemon behaves correctly
  *if* the real tool does too. Confirm empirically with the `### Verification` block below;
  if the real answer turns out to be "no, or much slower," the fix is a resume-shaped branch
  on the registration wait, not a redesign — the mechanism this document describes stays the
  same either way.

### Verification

```bash
cargo test -p claude_daemon_core --test serve_test
```

Against a live daemon — the round trip this feature exists to make possible:

```bash
clr chat "remember the word pineapple"
ID=$( clr sessions --json | jq -r '.[0].session_id' )

clr daemon stop && clr daemon start        # every session released
clr chat "what word did I ask you to remember?"

# Same id, and it remembers. A different id means resume did not happen.
clr sessions --json | jq -r '.[0].session_id'
echo "was: $ID"
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/serve.rs` | `Daemon::spawn`, the spawner bound |
| source | `src/paths.rs` | Locating the transcript directory a cwd maps to |
| dep | `claude_storage_core` | Reads the transcripts the last-occupant lookup resolves against |
| doc | [005_session_registration.md](005_session_registration.md) | The registration wait a resumed spawn also goes through |
| doc | [010_session_reaping.md](010_session_reaping.md) | The first consumer — releasing an idle session |
| doc | [`child_supervisor/docs/feature/001_session_table.md`](../../../child_supervisor/docs/feature/001_session_table.md) | The table this re-keys |
| doc | `claude_runner/docs/feature/008_interactive_handoff.md` | The second consumer — releasing to an interactive client |
| test | `tests/serve_test.rs` | Dispatch against a real socket and real children |
