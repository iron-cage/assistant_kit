# src/

Core library implementation for `claude_daemon_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `baseline.rs` | Measuring and caching a conversation's fixed token floor |
| `client.rs` | Issuing one request to a running daemon — thin wrapper over `daemon_kit::client` |
| `context.rs` | Rendering a session's context summary from its transcript |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `paths.rs` | Lock, socket, and registry directory resolution |
| `protocol.rs` | Request wire type and the Claude-specific `SessionSummary`; `Response` re-exported from `daemon_kit` |
| `registration.rs` | Waiting for a spawned process to publish its conversation id |
| `serve.rs` | Request dispatch and one-request-per-connection serving — composes `daemon_kit` and `child_supervisor` |

Single-instance locking, socket binding, IPC framing, and the client, plus
hosted-session bookkeeping and output buffering, moved to the generic
`daemon_kit` and `child_supervisor` crates — see their own `src/readme.md`.
