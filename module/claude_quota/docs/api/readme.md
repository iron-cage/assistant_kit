# API Doc Entity

### Scope

- **Purpose**: Document the programmatic interface of the claude_quota library surface.
- **Responsibility**: Index of API doc instances covering the Anthropic HTTP endpoint clients.
- **In Scope**: The five endpoint clients (rate limits, OAuth usage, OAuth account, CLI roles, models), their parse/fetch split, shared `QuotaError`, and the shared timeout-hardened agent contract.
- **Out of Scope**: How consumers render or cache quota data (→ `claude_profile` `.usage` pipeline, `claude_profile_core` quota cache), token acquisition and refresh (→ `claude_profile_core` `docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Endpoints](001_endpoints.md) | Anthropic endpoint client contracts (cluster-level) | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |
