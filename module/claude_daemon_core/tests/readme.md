# tests/

Integration tests for the `claude_daemon_core` crate. Locks are taken against
real files, IPC framing runs over real readers, and the session table holds real
PTY-attached children — no mocks.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `paths_test.rs` | Runtime, lock, socket, and registry locations from an injected home |
| `lock_test.rs` | Single-instance contention, release on drop, parent creation, content preservation |
| `ipc_test.rs` | Line framing, EOF, CRLF, non-UTF-8, and the size cap that bounds the buffer |
| `protocol_test.rs` | Wire shape of every request and both response forms, and round-trips |
| `output_test.rs` | Output cursors, bounded retention, and character boundaries |
| `registration_test.rs` | Finding a spawned process's conversation id, and giving up when it never appears |
| `table_test.rs` | Conversation-id keying, replacement, removal, summary ordering, and teardown |
| `listener_test.rs` | Binding over wreckage, socket permissions, and leaving nothing behind |
| `serve_test.rs` | End-to-end dispatch: a real socket, the real client, real children |
