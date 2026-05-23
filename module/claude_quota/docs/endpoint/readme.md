# Endpoint Reference

### Scope

- **Purpose**: Authoritative reference for every Anthropic HTTP endpoint consumed by workspace crates or discoverable from the Claude Code binary.
- **Responsibility**: One file per endpoint — URL, method, required headers, request body, complete response schema with all known fields, error codes, and known limitations.
- **In Scope**: All endpoints callable or observed via `strings $(which claude) | grep "^https://"` — whether actively implemented or partially blocked.
- **Out of Scope**: Anthropic public inference API contract (→ Anthropic docs); internal Rust parsing implementation (→ `src/lib.rs`); caller command design (→ respective crate `docs/feature/`).

### Index

| ID | Endpoint | Method | Workspace caller | Status |
|----|----------|--------|-----------------|--------|
| [001](001_oauth_usage.md) | `api.anthropic.com/api/oauth/usage` | GET | `claude_quota::fetch_oauth_usage` | ✅ implemented |
| [002](002_oauth_account.md) | `api.anthropic.com/api/oauth/account` | GET | `claude_quota::fetch_oauth_account` | ✅ implemented |
| [003](003_v1_messages.md) | `api.anthropic.com/v1/messages` (rate-limit headers) | POST | `claude_quota::fetch_rate_limits` | ✅ implemented |
| [004](004_oauth_token.md) | `platform.claude.com/v1/oauth/token` | POST | `claude_auth::refresh_token` | ✅ implemented |
| [005](005_claude_cli_roles.md) | `api.anthropic.com/api/oauth/claude_cli/roles` | GET | — | 📄 documented only |
| [006](006_create_api_key.md) | `api.anthropic.com/api/oauth/claude_cli/create_api_key` | POST | — | 🔒 requires `org:create_api_key` scope |
| [007](007_metrics_enabled.md) | `api.anthropic.com/api/claude_code/organizations/metrics_enabled` | GET | — | 🔒 enterprise accounts only |
| [008](008_shared_session_transcripts.md) | `api.anthropic.com/api/claude_code_shared_session_transcripts` | POST | — | 🔒 scope insufficient |
| [009](009_cli_feedback.md) | `api.anthropic.com/api/claude_cli_feedback` | POST | — | 🔒 scope insufficient |
| [010](010_web_domain_info.md) | `api.anthropic.com/api/web/domain_info` | GET | — | 📄 documented only |

### Field Index

[account_field_index.md](account_field_index.md) — cross-endpoint field inventory: every account field from endpoints 001–005 organized by concept domain (user identity, billing, organization, token lifecycle), with overlap analysis and a coverage matrix.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/lib.rs` | `fetch_oauth_usage`, `fetch_oauth_account`, `fetch_rate_limits` — transport implementations |
| doc | `../../../../claude_profile/docs/feature/009_token_usage.md` | `.usage` command — consumes endpoints 001 + 002 |
| doc | `../../../../claude_profile/docs/feature/013_account_limits.md` | `.account.limits` command — consumes endpoint 003 |
