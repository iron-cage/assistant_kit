# claude_session_core

Pure library for observing live Claude Code sessions.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation |
| `tests/` | Test suite for registry parsing, liveness, and turn detection |
| `docs/` | Behavioral requirements: features, invariants, api, algorithms |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

Claude Code maintains a mutable, PID-keyed registry at `~/.claude/sessions/`,
writing one `<pid>.json` per running process and reaping it on exit. This crate
reads that registry, decides whether the process behind a record is genuinely
running, and converts status transitions into turn boundaries.

It is deliberately **not** `claude_storage_core`. That crate owns the append-only
conversation transcripts under `~/.claude/projects/` — permanent data, a
different directory, a different format, and a different failure model. The two
stores are joined by the `sessionId` field carried in every registry record.

## features

- **Injected registry directory**: every entry point takes the directory as a
  parameter, so tests point at a `TempDir` and the crate stays at Layer 0
- **Incarnation-checked liveness**: `pid_alive` verifies stat readability,
  non-zombie state, thread-group leadership, and start-time match
- **Honest turn boundaries**: `idle` is not treated as "done" unless the session
  was spawned with background-task reporting enabled

## usage

```toml
[dependencies]
claude_session_core = { workspace = true }
```

```rust,no_run
use claude_session_core::{ scan_live, BackgroundReporting, TurnWatcher, TurnEvent };
use std::path::Path;

fn main() -> claude_session_core::Result< () >
{
  for record in scan_live( Path::new( "/home/me/.claude/sessions" ) )?
  {
    println!( "{} {} {:?}", record.pid, record.session_id, record.status );
  }

  let mut watcher = TurnWatcher::new( BackgroundReporting::Enabled );
  // ... feed statuses as they are observed ...
  if watcher.observe( &claude_session_core::SessionStatus::Busy ) == Some( TurnEvent::Started )
  {
    println!( "turn started" );
  }
  Ok( () )
}
```

## architecture

**Two traps this crate exists to encode.**

*A PID is not an identity.* `/proc/{pid}` existence proves only that a number is
in use. Unreaped zombies keep their entry; Linux resolves direct `/proc/<tid>`
lookups for readdir-invisible non-leader thread ids; a full PID-space wrap
recycles leader numbers. `liveness::pid_alive` carries all four clauses and the
two production defects that produced them.

*`idle` is not "done".* Claude Code's `Stop` hook payload carries a
`background_tasks` array precisely so hooks can tell a finished session from one
parked waiting to be woken. The registry's `status` field does not expose it, and
whether `status` even accounts for background work depends on
`CLAUDE_CODE_BG_TASKS_REPORT_RUNNING`, which defaults to off. `TurnWatcher` makes
that guarantee an explicit constructor argument, so an unverified boundary is
labelled `SettledUnverified` rather than silently trusted.
