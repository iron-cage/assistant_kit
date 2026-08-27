# docs/

### Scope

**Responsibilities:** Behavioral requirements for the `claude_auth` crate — the Anthropic OAuth token-refresh wire protocol: endpoint, client identity, request body, response parsing, and error classification.
**In Scope:** Capabilities (`feature/`), measurable constraints (`invariant/`), public API contracts (`api/`).
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`), and everything a consumer does with a refreshed token — quota accounting, account selection, credential storage, output formatting (→ `claude_profile`, `dream`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `entity.md` | Master registry of doc entities and instances |
| `feature/` | User-facing capabilities of the token-refresh transport |
| `invariant/` | Constraints that must hold for every build |
| `api/` | Public library API contracts |
