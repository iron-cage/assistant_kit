# Feature: Session Resume

### Scope

- **Purpose**: Start a hosted session attached to a conversation that already exists, rather than a fresh one — so releasing a session from the daemon stops meaning destroying the conversation inside it.
- **In Scope**: The spawner's resume parameter, the daemon's memory of which conversation last occupied a directory, and the conditions under which a spawn resumes instead of starting over.
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

- **Behaviour on a terminal.** The flags above are documented for the tool as a whole. That
  `--resume <id>` brings up a *usable interactive* session on a pty — rather than one that
  needs a keystroke first, the way the picker does — is **TBD** until observed against a real
  `claude`. Everything in this document assumes it does.
- **Whether a resumed session re-registers.** [005_session_registration.md](005_session_registration.md)
  waits for a conversation id to appear in the registry after a spawn. Whether a resumed
  session republishes its id, and how quickly, is **TBD**; if it does not, the registration
  wait needs a resume-shaped branch rather than the same timeout. Settled by a test during
  implementation rather than by inspection — the answer is cheaper to observe than to reason
  about.
- **Which transcript wins when a directory holds several.** Most-recently-modified is the
  obvious rule and probably right, but a directory that has hosted both a daemon session and
  an interactive one has two plausible answers. **TBD** whether the daemon should prefer the
  most recent unconditionally, or the most recent it previously hosted.

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
| source | `src/table.rs` | The cwd → conversation id map |
| doc | [005_session_registration.md](005_session_registration.md) | The registration wait a resumed spawn also goes through |
| doc | [010_session_reaping.md](010_session_reaping.md) | The first consumer — releasing an idle session |
| doc | [003_session_table.md](003_session_table.md) | The table this re-keys |
| doc | `claude_runner/docs/feature/008_interactive_handoff.md` | The second consumer — releasing to an interactive client |
| test | `tests/serve_test.rs` | Dispatch against a real socket and real children |
