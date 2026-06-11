# Verb :: status

Reports the current state of a live credential or token as a formatted snapshot. Applied to `token`, returns expiry classification (`Valid`, `ExpiringSoon`, or `Expired`). Applied to `credentials`, returns live OAuth credential metadata with per-field presence control. Both operations are pure reads from `~/.claude/.credentials.json` with no side effects.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [token](../command_noun/002_token.md) | `.token.status` | Yes | Yes |
| 2 | [credentials](../command_noun/003_credentials.md) | `.credentials.status` | Yes | Yes |

### Behavioral Contract

**Pre-conditions:**
- `~/.claude/.credentials.json` readable
- `$HOME` environment variable set
- For `.token.status`: `expiresAt` field present and parseable in credentials file
- For `.credentials.status`: active session may optionally have supplementary files (`~/.claude.json`, `~/.claude/settings.json`, `{active_name}.json`) for extended fields

**Post-conditions:**
- Credential or token state reported (read-only)
- No files written or modified

**Side effects:**
- `.credentials.status` may perform supplementary reads from `~/.claude.json`, `~/.claude/settings.json`, and `{active_name}.json` when corresponding field-presence params are enabled
- No writes, no network requests

### Idempotency

**Yes.** Both commands are pure reads. Repeated calls return the same result for the same credential state. No side effects accumulate.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `format::` | Output format (`text` or `json`) — both commands | No |
| `threshold::` | Warning threshold in seconds for `ExpiringSoon` classification (`.token.status` only) | No |
| `trace::` | Emit diagnostic trace output — both commands | No |

Field-presence parameters for `.credentials.status` (`account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) each toggle inclusion of the corresponding field in text output. `format::json` always includes all fields regardless.

### State Transition Pattern

**Reads state.** Both commands read `~/.claude/.credentials.json` and optionally supplementary files. No local writes. Credential and account lifecycle state unchanged.

```
[active] --token.status-------> [active]        (expiry classification read; no change)
[active] --credentials.status-> [active]        (credential metadata read; no change)
```

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/006_token_status.md](../../feature/006_token_status.md) | Token expiry classification algorithm and threshold semantics |
| [feature/012_live_credentials_status.md](../../feature/012_live_credentials_status.md) | Live credential read algorithm and field sourcing |
| [feature/014_rich_account_metadata.md](../../feature/014_rich_account_metadata.md) | Extended metadata fields from `~/.claude.json` and `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Classify active OAuth token expiry state |
| 2 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Show live credential metadata with field presence control |
