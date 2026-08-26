# src/

Core library implementation for `claude_daemon_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `client.rs` | Issuing one request to a running daemon |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `ipc.rs` | Size-capped line framing for the socket protocol |
| `listener.rs` | Socket binding, stale-socket removal, cleanup on drop |
| `lock.rs` | Advisory `flock` single-instance enforcement |
| `output.rs` | Bounded, cursor-addressed session output and its pump thread |
| `paths.rs` | Lock, socket, and registry directory resolution |
| `protocol.rs` | Request and response wire types |
| `registration.rs` | Waiting for a spawned process to publish its conversation id |
| `render.rs` | Rendering raw terminal output as readable plain text |
| `serve.rs` | Request dispatch and one-request-per-connection serving |
| `table.rs` | Hosted-session table keyed by conversation id |
