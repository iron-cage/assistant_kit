# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_daemon_core` crate — the single-instance guarantee, the hosted-session table, and the line-framed request/response protocol clients speak to the daemon.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), terminal mechanics (→ `claude_pty_core`), liveness and turn detection (→ `claude_session_core`), the CLI that drives the daemon (→ `claude_runner`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the daemon layer |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
