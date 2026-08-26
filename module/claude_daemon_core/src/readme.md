# src/

Core library implementation for `claude_daemon_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `ipc.rs` | Size-capped line framing for the socket protocol |
| `lock.rs` | Advisory `flock` single-instance enforcement |
| `paths.rs` | Lock, socket, and registry directory resolution |
| `protocol.rs` | Request and response wire types |
| `table.rs` | Hosted-session table keyed by conversation id |
