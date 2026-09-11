# src/

Core library implementation for `daemon_kit`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `client.rs` | Issuing one request to a running daemon over its socket |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `ipc.rs` | Size-capped line framing for the socket protocol |
| `listener.rs` | Socket binding, stale-socket removal, cleanup on drop |
| `lock.rs` | Advisory `flock` single-instance enforcement |
| `response.rs` | Payload-agnostic `{ok, result}` / `{ok, error}` envelope |
| `serve.rs` | One request/response exchange, and the accept-loop body it lives in |
