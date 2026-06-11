# Credentials

The live OAuth credential metadata for the active Claude Code session, read directly from `~/.claude/.credentials.json`. Provides identity and session information without requiring account store setup — succeeds on any authenticated machine regardless of whether named account profiles exist.

### Commands

| # | Command | Verb | Purpose | Idempotent | Requires Session |
|---|---------|------|---------|-----------|-----------------|
| 1 | `.credentials.status` | status | Show live credential metadata with per-field presence control | Yes | Yes |

### Parameter Matrix

| Parameter | `.credentials.status` |
|-----------|----------------------|
| `format::` | optional |
| `account::` | optional |
| `sub::` | optional |
| `tier::` | optional |
| `token::` | optional |
| `expires::` | optional |
| `email::` | optional |
| `file::` | optional |
| `saved::` | optional |
| `display_name::` | optional |
| `role::` | optional |
| `billing::` | optional |
| `model::` | optional |
| `uuid::` | optional |
| `capabilities::` | optional |
| `org_uuid::` | optional |
| `org_name::` | optional |
| `trace::` | optional |

### Lifecycle

Stateless — credentials operations are pure reads with no persistent state.

The credential lifecycle is managed externally: `clp` reads but never writes `~/.claude/.credentials.json` via `.credentials.status`. The file is written by `claude` OAuth login, `.account.use`, and `.account.relogin`.

### Provider Contract

| Operation | Implementation |
|-----------|---------------|
| `.credentials.status` | `credentials::read_live()` — reads `~/.claude/.credentials.json` + per-enabled-field supplementary reads from `~/.claude.json`, `~/.claude/settings.json`, `{active_name}.json` |

### Output Schema

**`.credentials.status format::json`:**

```json
{
  "account": "alice@acme.com",
  "subscription": "max",
  "tier": "default_claude_max_20x",
  "token": "valid",
  "expires_in_secs": 26640,
  "email": "alice@acme.com",
  "file": "/home/user/.claude/.credentials.json",
  "saved": 2,
  "display_name": "alice",
  "role": "admin",
  "billing": "stripe_subscription",
  "model": "sonnet",
  "tagged_id": "user_01ABCdef",
  "capabilities": ["claude_max", "chat"],
  "organization_uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "organization_name": "alice@example.com's Organization"
}
```

`format::json` always includes all fields regardless of field-presence params.

### Error Codes

| Code | Trigger | Recovery |
|------|---------|---------|
| exit 2 | `~/.claude/.credentials.json` absent | Authenticate via `claude` binary |
| exit 2 | `$HOME` env variable unset | Set `$HOME` before invoking `clp` |

### Relationships

| Related Entity | Relationship | Direction |
|----------------|-------------|---------|
| `account` | Credentials reflect the active account; account store is not required but supplements supplementary fields | credentials → account (optional) |
| `token` | Credentials contain the OAuth token; token status is derived from the same credential file | credentials → token |

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/012_live_credentials_status.md](../../feature/012_live_credentials_status.md) | Live credential read algorithm and field sourcing |
| [feature/014_rich_account_metadata.md](../../feature/014_rich_account_metadata.md) | Extended metadata fields from `~/.claude.json` and `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Show live credential metadata |
