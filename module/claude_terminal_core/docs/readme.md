# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_terminal_core` crate — interpreting a terminal's escape-sequence byte stream as the plain text a reader would have seen.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), allocating a terminal and spawning onto it (→ `claude_pty_core`), where a hosted session's output is retained and how a client asks for it (→ `claude_daemon_core`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the rendering layer |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
