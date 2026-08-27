# src/

Dependency-light HTTP client for five Anthropic API endpoints.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Five endpoint clusters (rate limits, OAuth usage, OAuth account, CLI roles, models), each split into a pure `parse_*` function and a `fetch_*` function, plus the shared `QuotaError` type |

### Scope

**In Scope:**
- Parsing and fetching: rate-limit headers, OAuth usage, OAuth account, CLI roles, model catalog
- Hardened shared HTTP agent (`https_only`, timeouts) for all network calls (feature `enabled`)

**Out of Scope:**
- OAuth token acquisition/refresh (→ `claude_auth`; callers supply a valid `token: &str`)
- Quota caching and rendering (→ consumer crates)
- Retry policy (consumers decide; `QuotaError::HttpStatus` gives a stable signal)

See [`docs/api/001_endpoints.md`](../docs/api/001_endpoints.md) for the full behavioral contract per cluster.
