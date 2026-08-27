# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_topic_core` crate — what a topic name resolves to, which topics exist, which one a prompt should go to, and how two writers are kept off one conversation.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), session paths and the `UUIDv5` rule this crate calls into (→ `claude_storage_core`), the process scan selection is judged against (→ `claude_core`), pid liveness (→ `claude_session_core`), actually invoking Claude Code for a topic (→ `claude_runner`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the topic layer |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
