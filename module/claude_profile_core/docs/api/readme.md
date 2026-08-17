# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the claude_profile_core library surface.
- **Responsibility**: Index of API doc instances covering the `token` and `account` module contracts.
- **In Scope**: `token` expiry classification functions and the `TokenStatus` type; `account` credential-store domain logic (CRUD, switch, refresh, ownership, quota cache, history).
- **Out of Scope**: CLI binary behavior (this crate has no binary — `clp` lives in `claude_profile`), the shared atomic-write/settings primitives (→ `claude_core` `docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Token](001_token.md) | OAuth token expiry classification contract | ✅ |
| 002 | [Account](002_account.md) | Credential-store account domain logic contract (cluster-level) | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
