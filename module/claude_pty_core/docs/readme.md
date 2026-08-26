# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_pty_core` crate — pseudo-terminal allocation, child spawning onto a PTY, and the writer thread that keeps a caller off the master descriptor.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), everything above the terminal layer — liveness (→ `claude_session_core`), the daemon and its protocol (→ `claude_daemon_core`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the PTY layer |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
