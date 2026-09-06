# tests/

Integration tests for the `claude_daemon_core` crate — the Claude-specific
remainder after `daemon_kit` (generic daemon skeleton) and `child_supervisor`
(generic session bookkeeping) were split out. Locks, sockets, IPC framing, and
session-table mechanics are exercised in those crates' own suites now; what's
here is real PTY-attached children driven through the actual wire protocol —
no mocks.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `paths_test.rs` | Runtime, lock, socket, and registry locations from an injected home |
| `protocol_test.rs` | Wire shape of every request and `SessionSummary`, and round-trips |
| `context_test.rs` | Context summary folded from a transcript: every section, the token split, missing transcripts, unmodelled kinds, and that the read stays pure |
| `baseline_test.rs` | Probe flags, reading a probe response, and the version-and-model-keyed cache |
| `registration_test.rs` | Finding a spawned process's conversation id, and giving up when it never appears |
| `serve_test.rs` | End-to-end dispatch: a real socket, the real client, real children |
