# Endpoint Doc Entity

### Scope

- **Purpose**: Authoritative wire contract for every Anthropic HTTP endpoint consumed by or observable from workspace crates.
- **Responsibility**: Master file for the `endpoint` collection — lists all 11 instances, declares scope boundaries, and links to supplementary cross-endpoint reference material.
- **In Scope**: All endpoints callable or observed via `strings $(which claude)` — literal full URLs (`grep "^https://"`) plus relative-path client calls (see § Detecting an Endpoint below) — whether actively implemented or partially blocked.
- **Out of Scope**: Anthropic public inference API contract (→ Anthropic docs); internal Rust parsing implementation (→ respective crate `src/lib.rs`); caller command design (→ respective crate `docs/feature/`).

### Overview Table

| ID | Name | URL | Method | Workspace Caller | Status |
|----|------|-----|--------|-----------------|--------|
| [001](001_oauth_usage.md) | OAuth Usage | `api.anthropic.com/api/oauth/usage` | GET | `claude_quota::fetch_oauth_usage` | ✅ implemented |
| [002](002_oauth_account.md) | OAuth Account | `api.anthropic.com/api/oauth/account` | GET | `claude_quota::fetch_oauth_account` | ✅ implemented |
| [003](003_v1_messages.md) | Messages Rate-Limit Headers | `api.anthropic.com/v1/messages` | POST | `claude_quota::fetch_rate_limits` | ✅ implemented |
| [004](004_oauth_token.md) | OAuth Token Refresh | `platform.claude.com/v1/oauth/token` | POST | `claude_auth::refresh_token` | ✅ implemented |
| [005](005_claude_cli_roles.md) | Claude CLI Roles | `api.anthropic.com/api/oauth/claude_cli/roles` | GET | `claude_quota::fetch_claude_cli_roles` | ✅ implemented |
| [006](006_create_api_key.md) | Create API Key | `api.anthropic.com/api/oauth/claude_cli/create_api_key` | POST | — | 🔒 requires `org:create_api_key` scope |
| [007](007_metrics_enabled.md) | Metrics Enabled | `api.anthropic.com/api/claude_code/organizations/metrics_enabled` | GET | — | 🔒 enterprise accounts only |
| [008](008_shared_session_transcripts.md) | Shared Session Transcripts | `api.anthropic.com/api/claude_code_shared_session_transcripts` | POST | — | 🔒 scope insufficient |
| [009](009_cli_feedback.md) | CLI Feedback | `api.anthropic.com/api/claude_cli_feedback` | POST | — | 🔒 scope insufficient |
| [010](010_web_domain_info.md) | Web Domain Info | `api.anthropic.com/api/web/domain_info` | GET | — | 📄 documented only |
| [011](011_v1_models.md) | List Models | `api.anthropic.com/v1/models` | GET | `claude_quota::fetch_models` | ✅ implemented |

### Detecting an Endpoint

The scope-declaring technique above (`grep "^https://"`) only finds endpoints called with a full literal URL string. It misses any endpoint called through a pre-configured base-URL HTTP client — `Mi.get(path)`, `Mi.post(path)`, `Mi.patch(path)` in the bundled JS, where `path` is a relative fragment joined against a base URL set once elsewhere. Five of this collection's 11 documented endpoints (001, 002, 007, 008, 009) are only observable this way; the anchored `^https://` grep returns zero matches for all five.

The fuller technique: after the anchored grep, also run a looser, unanchored fragment search for each candidate endpoint's own path segment, e.g.:

```bash
strings $(which claude) | grep -o '.\{0,20\}oauth/usage.\{0,20\}'
```

This surfaced two further sub-paths not documented as their own instances: `/api/oauth/account/settings` (GET+PATCH) and `/api/oauth/account/grove_notice_viewed` (POST). Neither is called from `module/claude_quota/src/` or `module/claude_auth/src/` (confirmed via direct grep of both crates). Per this file's own scope declaration ("consumed by or observable from workspace crates"), they are observable but not consumed — documenting their request/response schema would mean fabricating it without live API evidence. Left out of the Overview Table for that reason, not omitted by oversight.

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

### Cross-Collection Dependencies

**Implemented by**:
- `module/claude_quota/src/lib.rs` — endpoints 001, 002, 003, 005, 011
- `module/claude_auth/src/lib.rs` — endpoint 004

**Consumed by**:
- `module/claude_profile/docs/feature/009_token_usage.md` — `.usage` command (endpoints 001 + 002)
- `module/claude_profile/docs/feature/013_account_limits.md` — `.account.limits` command (endpoint 003)
- `module/claude_profile/docs/feature/017_token_refresh.md` — refresh policy (endpoint 004)
- `module/claude_profile/docs/feature/022_org_identity_snapshot.md` — `.account.save` org snapshot (endpoint 005)
- `module/claude_profile/docs/feature/068_models_list_command.md` — `.models` command (endpoint 011)
