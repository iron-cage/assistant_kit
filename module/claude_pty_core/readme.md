# claude_pty_core

Pure library for pseudo-terminal session mechanics (zero dependencies).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation |
| `tests/` | Test suite for PTY allocation, writer queue, and env scrubbing |
| `docs/` | Behavioral requirements: features, invariants, api, algorithms |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

Allocates pseudo-terminals and spawns child processes attached to them. Knows
nothing about Claude Code — it is a generic terminal-mechanics layer that higher
crates compose. Zero runtime dependencies.

## features

- **Zero dependencies**: hand-rolled POSIX FFI, no `libc`/`nix`/`rustix` crate
- **Scoped unsafe**: every `extern "C"` declaration and `unsafe` block is confined
  to `src/ffi.rs` under `#[ allow( unsafe_code ) ]` with a `SAFETY:` justification
- **Non-blocking writes**: a dedicated writer thread with a bounded queue, so a
  child that stops reading stdin can never stall the caller's event loop
- **Terminal identity scrubbing**: strips inherited terminal control handles so a
  spawned agent cannot reach back into the host's multiplexer session

## usage

```toml
[dependencies]
claude_pty_core = { workspace = true }
```

```rust,no_run
use claude_pty_core::{ PtySession, SessionConfig, WinSize };

fn main() -> claude_pty_core::Result< () >
{
  let config = SessionConfig::new( "claude" )
    .arg( "--version" )
    .win_size( WinSize::new( 40, 120 ) );
  let mut session = PtySession::spawn( &config )?;
  session.write( b"hello\r" )?;
  session.resize( WinSize::new( 50, 160 ) )?;
  session.shutdown()?;
  Ok( () )
}
```

## architecture

**Why hand-rolled FFI**: `pty-process` 0.5.3 declares `edition = "2024"`, which
requires Rust 1.85 — the workspace pins `rust-version = "1.75"`. `portable-pty`
0.9.0 is edition-compatible but exposes `anyhow::Error` in its public API, which
collides with this project's error_tools-exclusive rule, and pulls 18 transitive
packages. Allocating a PTY via `posix_openpt`/`grantpt`/`unlockpt`/`ptsname_r` is
roughly 80 lines and keeps the zero-dependency guarantee.

**Why a writer thread**: writing to a PTY master blocks once the kernel input
buffer fills, which happens whenever the child stops reading stdin — busy, hung,
or stopped. A synchronous `write_all` from a caller's event loop would then stall
everything. Writes are queued to a bounded channel and drained by a dedicated
thread; a full queue reports `Error::WriterFull` rather than growing without limit.
