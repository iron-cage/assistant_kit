# docs/

### Scope

**Responsibilities:** API contract for the `claude_auth` crate.
**In Scope:** OAuth token-refresh public API (`TokenRefreshResult`, `AuthError`, `parse_response`, `refresh_token`).
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), caller-specific wiring (-> consuming crates' own docs).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `api/` | Public library API contract: TokenRefreshResult, AuthError, parse_response, refresh_token |
