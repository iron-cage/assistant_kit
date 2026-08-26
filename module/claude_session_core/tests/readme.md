# tests/

Integration tests for the `claude_session_core` crate. Registry tests use real
files in temporary directories; liveness tests use real processes, including a
real unreaped zombie — no mocks.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `registry_test.rs` | Record parsing, `procStart`-as-string, missing directory, torn writes, ordering |
| `liveness_test.rs` | The four liveness clauses, including a real zombie and a real thread id |
| `turn_test.rs` | Edge-triggered transitions, first-sighting rule, background-reporting labelling |
