# tests/

Integration tests for the `claude_terminal_core` crate. Every case is a literal
string in and an exact string out — no mocks, and no `contains` assertions.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `render_test.rs` | Escape removal, in-line cursor motion, and what survives trimming |
