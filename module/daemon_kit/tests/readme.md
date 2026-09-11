# tests/

Integration tests for the `daemon_kit` crate. Locks are taken against real
files, sockets are real Unix domain sockets, and IPC framing runs over real
readers — no mocks.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lock_test.rs` | Single-instance contention, release on drop, parent creation, content preservation |
| `listener_test.rs` | Binding over wreckage, socket permissions, and leaving nothing behind |
| `ipc_test.rs` | Line framing, EOF, CRLF, non-UTF-8, and the size cap that bounds the buffer |
| `response_test.rs` | Wire shape of both response forms, the `ok` discriminant, and round-trips |
