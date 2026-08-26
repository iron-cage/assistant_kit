# tests/

Integration tests for the `claude_pty_core` crate. Every test uses a real
pseudo-terminal and, where a child is needed, a real process — no mocks.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `pty_test.rs` | Allocation, slave-path shape, independent slave descriptors, resize |
| `session_test.rs` | Spawn, write/read round-trip, controlling terminal, cwd, lifecycle |
| `writer_test.rs` | Delivery ordering, capacity rejection, shutdown and drop behavior |
| `env_scrub_test.rs` | Scrub-list membership, `CLAUDE_` prefix rule, `TERM` replacement |
| `unsafe_containment_test.rs` | `unsafe` appears in no module but `src/ffi.rs` |
