# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `child_supervisor` crate — the hosted-session table and the drained, cursor-addressed output buffer beneath it.
**In Scope:** Capabilities (`feature/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), terminal mechanics (→ `claude_pty_core`), the daemon that composes this crate (→ `claude_daemon_core`), invariant and API doc instances shared with that daemon (→ `claude_daemon_core/docs/invariant/`, `claude_daemon_core/docs/api/` — this crate has no doc instances of its own in either category yet).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the session-supervision layer |
