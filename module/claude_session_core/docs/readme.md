# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_session_core` crate — reading Claude Code's live-session registry, deciding whether a recorded process is genuinely alive, and detecting turn boundaries from observed status.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), conversation transcripts (→ `claude_storage_core`), terminal mechanics (→ `claude_pty_core`), the daemon that consumes this crate (→ `claude_daemon_core`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the session-observation layer |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
