# Endpoint Reference

### Scope

- **Purpose**: Authoritative wire contract for every Anthropic HTTP endpoint consumed by or observable from workspace crates.
- **Responsibility**: Master file for the `endpoint` doc entity — lists all 10 instances, declares scope boundaries, and links to supplementary cross-endpoint reference material.
- **In Scope**: All endpoints callable or observed via `strings $(which claude) | grep "^https://"` — whether actively implemented or partially blocked.
- **Out of Scope**: Anthropic public inference API contract (→ Anthropic docs); internal Rust parsing implementation (→ respective crate `src/lib.rs`); caller command design (→ respective crate `docs/feature/`).

### Overview Table

| ID | Name | URL | Method | Workspace Caller | Status |
|----|------|-----|--------|-----------------|--------|
| [001](001_oauth_usage.md) | OAuth Usage | `api.anthropic.com/api/oauth/usage` | GET | `claude_quota::fetch_oauth_usage` | ✅ implemented |
| [002](002_oauth_account.md) | OAuth Account | `api.anthropic.com/api/oauth/account` | GET | `claude_quota::fetch_oauth_account` | ✅ implemented |
| [003](003_v1_messages.md) | Messages Rate-Limit Headers | `api.anthropic.com/v1/messages` | POST | `claude_quota::fetch_rate_limits` | ✅ implemented |
| [004](004_oauth_token.md) | OAuth Token Refresh | `platform.claude.com/v1/oauth/token` | POST | `claude_auth::refresh_token` | ✅ implemented |
| [005](005_claude_cli_roles.md) | Claude CLI Roles | `api.anthropic.com/api/oauth/claude_cli/roles` | GET | — | 📄 documented only |
| [006](006_create_api_key.md) | Create API Key | `api.anthropic.com/api/oauth/claude_cli/create_api_key` | POST | — | 🔒 requires `org:create_api_key` scope |
| [007](007_metrics_enabled.md) | Metrics Enabled | `api.anthropic.com/api/claude_code/organizations/metrics_enabled` | GET | — | 🔒 enterprise accounts only |
| [008](008_shared_session_transcripts.md) | Shared Session Transcripts | `api.anthropic.com/api/claude_code_shared_session_transcripts` | POST | — | 🔒 scope insufficient |
| [009](009_cli_feedback.md) | CLI Feedback | `api.anthropic.com/api/claude_cli_feedback` | POST | — | 🔒 scope insufficient |
| [010](010_web_domain_info.md) | Web Domain Info | `api.anthropic.com/api/web/domain_info` | GET | — | 📄 documented only |

### Supplementary Reference

- [account_field_index.md](account_field_index.md) — cross-endpoint field inventory: every account field from endpoints 001–005 organized by concept domain (user identity, billing, organization, token lifecycle), with overlap analysis and a coverage matrix.

### Type-Specific Requirements

All `endpoint` doc instances must include:

1. **Title**: `# Endpoint: {METHOD} {path}` — using the HTTP method and URL path
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Request** (H3): URL, required headers, body (if any)
4. **Response** (H3): HTTP status, full JSON schema table with all known fields
5. **Error Codes** (H3): HTTP status code table
6. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Doc Entity Dependencies

**Implemented by**:
- `module/claude_quota/src/lib.rs` — endpoints 001, 002, 003
- `module/claude_auth/src/lib.rs` — endpoint 004

**Consumed by**:
- `module/claude_profile/docs/feature/009_token_usage.md` — `.usage` command (endpoints 001 + 002)
- `module/claude_profile/docs/feature/013_account_limits.md` — `.account.limits` command (endpoint 003)
- `module/claude_profile/docs/feature/017_token_refresh.md` — refresh policy (endpoint 004)
