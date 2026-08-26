# claude_daemon_core

Pure library for the single-instance session daemon and its IPC protocol.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation |
| `tests/` | Test suite for protocol round-trips, IPC framing, and instance locking |
| `docs/` | Behavioral requirements: features, invariants, api, data structures |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

Owns disowned interactive Claude Code sessions. Exactly one daemon runs at a
time — enforced by an advisory `flock`, not a PID file — and it hosts any number
of sessions, each an interactive `claude` process on its own pseudo-terminal.

Clients talk to it over a Unix domain socket, one JSON object per line.

## features

- **Single instance, many sessions**: an advisory `flock` the kernel releases on
  crash, so a stale lock cannot outlive its owner
- **Conversation-id addressing**: sessions survive a `--fork-session` re-host,
  which changes the PID
- **Capped protocol framing**: an unterminated line is refused at 1 MiB rather
  than allocated without bound
- **Composes, does not absorb**: PTY mechanics live in `claude_pty_core`,
  liveness and turn detection in `claude_session_core`

## architecture

**Why not a PID file.** A PID file records an intention and enforces nothing. A
daemon killed with `SIGKILL` leaves behind a file naming a PID that may since have
been recycled to an unrelated process, so the next start either refuses to run or
adopts a stranger. An advisory `flock` is released by the kernel when the last
descriptor closes — including on crash — so the lock cannot outlive its holder.

**Why conversation ids, not PIDs.** Claude Code's own daemon re-hosts a live
session with `--fork-session` on auto-update or recovery. The replacement process
has a different PID, no inherited environment, and a new conversation id. Every
identity scheme built on process inheritance breaks at that moment — which is
precisely when recovery was meant to help. The conversation id is the handle a
client keeps holding.

**Why the line cap.** The `clr query` prototype this generalizes reads its socket
with a bare `read_line`, which grows until a newline arrives. That was survivable
when each session had its own helper process; with one daemon hosting everything,
a peer that never sends a newline takes down every session at once.
