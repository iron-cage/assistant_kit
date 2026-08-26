# src/

Core library implementation for `claude_session_core`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root, module wiring, public re-exports |
| `error.rs` | Hand-rolled error type and crate `Result` alias |
| `liveness.rs` | `/proc`-based PID liveness and incarnation checking |
| `registry.rs` | `~/.claude/sessions/` record parsing and scanning |
| `turn.rs` | Turn-boundary detection over status transitions |
