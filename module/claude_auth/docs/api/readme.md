# API Doc Entity

### Scope

- **Purpose**: Pin the public library contract of `claude_auth` so consumers can depend on it without reading the source.
- **Responsibility**: Index of API doc instances covering signatures, error contracts, and availability by feature.
- **In Scope**: Public constants, types, and functions exported from `lib.rs`.
- **Out of Scope**: Behavioral rationale (→ `feature/`), structural constraints (→ `invariant/`), private helpers.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Auth Surface](001_auth_surface.md) | Every exported constant, type, and function with its contract | ✅ |
| 002 | [Token Refresh API](002_token_refresh_api.md) | TokenRefreshResult, AuthError, parse_response, refresh_token contract | ⚠️ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |

> **Pending consolidation.** 001 and 002 arrived from two branches and document the *same*
> surface — every exported constant, type, and function of `lib.rs`. They are not a split by
> subject; they are two drafts of one contract. 002 carries `Status`/`Since` metadata that 001
> lacks; 001 is the instance the `feature/` and `invariant/` cross-references point at. Only
> one should survive, or they should be folded into one — that decision is open. 002 was
> renumbered from 001 solely to keep this table's IDs unique; the renumbering implies nothing
> about which instance wins.
