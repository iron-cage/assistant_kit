# child_supervisor

Generic PTY-child bookkeeping. Knows nothing about what the child process is or
what protocol a caller speaks to it.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/output.rs` | Bounded, cursor-addressed terminal output, kept drained |
| `src/table.rs` | Keyed session table and per-session lifecycle (write/resize/read/shutdown) |
| `src/error.rs` | Crate-wide error type |
| `tests/` | Output buffer/pump behavior and session table lifecycle |

## overview

Extracted from `claude_daemon_core` (see its `docs/feature/readme.md` for the
retirement note on IDs 003/004, and this crate's own `docs/feature/` for the
relocated documents) — this crate is the mechanism those documents describe,
minus anything Claude-specific. A caller supplies its own session id scheme and
drives busy/idle bookkeeping itself; everything else — draining a pty master
into a bounded buffer, cursor-addressed reads, and the keyed table tying a
session id to a live child — is here.

## composes into

`claude_daemon_core` depends on this crate for the parts of its session
bookkeeping that have nothing to do with the Claude-specific wire protocol
built on top of it.
