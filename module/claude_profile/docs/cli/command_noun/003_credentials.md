# Noun: credentials

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

### See Also

| File | Relationship |
|------|-------------|
| [feature/012_live_credentials_status.md](../../feature/012_live_credentials_status.md) | Live credential read algorithm and field sourcing |
| [feature/014_rich_account_metadata.md](../../feature/014_rich_account_metadata.md) | Extended metadata fields from `~/.claude.json` and `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Show live credential metadata |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format::`](../param/002_format.md) | Output serialization format |
| 2 | [`account::`](../param/005_account.md) | Show active account name line |
| 3 | [`sub::`](../param/006_sub.md) | Show subscription type line |
| 4 | [`tier::`](../param/007_tier.md) | Show rate-limit tier line |
| 5 | [`token::`](../param/008_token.md) | Show token status line |
| 6 | [`expires::`](../param/009_expires.md) | Show token expiry duration line |
| 7 | [`email::`](../param/010_email.md) | Show email address line |
| 8 | [`file::`](../param/011_file.md) | Show credentials file path line |
| 9 | [`saved::`](../param/012_saved.md) | Show saved account count line |
| 10 | [`display_name::`](../param/014_display_name.md) | Show display name line |
| 11 | [`role::`](../param/015_role.md) | Show organisation role line |
| 12 | [`billing::`](../param/016_billing.md) | Show billing type line |
| 13 | [`model::`](../param/017_model.md) | Show active model line |
| 14 | [`uuid::`](../param/028_uuid.md) | Show stable user ID line |
| 15 | [`capabilities::`](../param/029_capabilities.md) | Show product capabilities list line |
| 16 | [`org_uuid::`](../param/030_org_uuid.md) | Show organisation UUID line |
| 17 | [`org_name::`](../param/031_org_name.md) | Show organisation display name line |
| 18 | [`trace::`](../param/023_trace.md) | Diagnostic trace output |
