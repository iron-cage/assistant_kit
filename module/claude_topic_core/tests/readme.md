# tests/

Integration tests for the `claude_topic_core` crate. Every case builds a real
temp directory tree, a real registry file, or a real lock file and asserts
against it — no mocks, and no test reaches the network or a live Claude Code
process.

The process-dependent paths are exercised through the same environment overrides
production uses: `CLR_TOPIC_REGISTRY_DIR` for the registry, `CLR_TOPIC_LOCK_DIR`
for locks, and `ProcessInfo` values constructed directly for selection — so a
test never depends on what happens to be running on the host.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `identity_test.rs` | Mode precedence, base resolution, name round-tripping |
| `registry_test.rs` | Append-if-missing, newline refusal, corrupt-file tolerance |
| `enumerate_test.rs` | Merged listing, sort order, the live filter |
| `select_test.rs` | Busy detection, idle preference, seeded determinism |
| `pool_test.rs` | Prefix validation, index parsing, idempotent top-up |
| `lock_test.rs` | Exclusion, drop release, dead-owner reclaim |
