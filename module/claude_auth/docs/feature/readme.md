# Feature Doc Entity

### Scope

- **Purpose**: Document behavioral requirements of the `claude_auth` token-refresh transport.
- **Responsibility**: Index of feature doc instances covering the OAuth exchange and its response parsing.
- **In Scope**: Endpoint and request construction, HTTP status classification, expiry computation, JSON field extraction.
- **Out of Scope**: Dependency and offline constraints (→ `invariant/`), signature contracts (→ `api/`), token consumption (→ `claude_profile`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Token Refresh](001_token_refresh.md) | OAuth exchange: endpoint, request body, status classification, expiry (FR-1–FR-9) | ✅ |
| 002 | [Response Parsing](002_response_parsing.md) | Dependency-free JSON field extraction and error attribution (FR-1–FR-7) | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
