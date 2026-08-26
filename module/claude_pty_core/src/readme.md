# src/

Core library implementation for `claude_pty_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `ffi.rs` | Every `extern "C"` declaration and `unsafe` block in the crate |
| `pty.rs` | `Pty` master/slave pair allocation, `WinSize`, resize |
| `env_scrub.rs` | Terminal-identity and Claude-marker environment scrubbing |
| `writer.rs` | Bounded-queue writer thread that never blocks the caller |
| `session.rs` | `PtySession` — composes pty, child process, and writer |
