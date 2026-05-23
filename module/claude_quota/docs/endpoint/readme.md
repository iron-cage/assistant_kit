# Endpoint Reference

### Scope

- **Purpose**: Authoritative reference for every Anthropic HTTP endpoint consumed by workspace crates.
- **Responsibility**: One file per endpoint — URL, method, required headers, request body, complete response schema with all known fields, error codes, and known limitations.
- **In Scope**: Endpoints actively called or confirmed usable by workspace code (`/api/oauth/usage`, `/api/oauth/account`, `POST /v1/messages` rate-limit side-channel).
- **Out of Scope**: Anthropic public inference API contract (→ Anthropic docs); internal Rust parsing implementation (→ `src/lib.rs`); caller command design (→ respective crate `docs/feature/`).

### Index

| ID | Endpoint | Method | Workspace caller | Status |
|----|----------|--------|-----------------|--------|
| [001](001_oauth_usage.md) | `/api/oauth/usage` | GET | `claude_quota::fetch_oauth_usage` | ✅ |
| [002](002_oauth_account.md) | `/api/oauth/account` | GET | `claude_quota::fetch_oauth_account` | ✅ |
| [003](003_v1_messages.md) | `/v1/messages` (rate-limit headers) | POST | `claude_quota::fetch_rate_limits` | ✅ |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/lib.rs` | `fetch_oauth_usage`, `fetch_oauth_account`, `fetch_rate_limits` — transport implementations |
| doc | `../../../../claude_profile/docs/feature/009_token_usage.md` | `.usage` command — consumes endpoint 001 |
| doc | `../../../../claude_profile/docs/feature/013_account_limits.md` | `.account.limits` command — consumes endpoint 003 |
