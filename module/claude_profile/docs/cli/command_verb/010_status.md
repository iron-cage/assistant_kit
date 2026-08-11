# Verb: status

Reports the current state of a live credential as a formatted snapshot via `credentials`, including OAuth token expiry classification (`Valid`, `ExpiringSoon`, or `Expired`) folded into its `token`/`expires` fields. The former standalone `token` noun's `.token.status` command has been removed (absorbed into `.credentials.status`). Reads are pure — from `~/.claude/.credentials.json` with no side effects.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [token](../command_noun/002_token.md) | `.token.status` — **REMOVED**, see `.credentials.status` | — | — |
| 2 | [credentials](../command_noun/003_credentials.md) | `.credentials.status` | Yes | Yes |

### Behavioral Contract

**Pre-conditions:**
- `~/.claude/.credentials.json` readable
- `$HOME` environment variable set
- `expiresAt` field present and parseable in credentials file (for the `Token:`/`Expires:` lines) — genuinely absent (not an error) for a `backend: redirect` account, which classifies as `Token: static`/`Expires: N/A` instead
- Active session may optionally have supplementary files (`~/.claude.json`, `~/.claude/settings.json`, `{active_name}.json`) for extended fields

**Post-conditions:**
- Credential or token state reported (read-only)
- No files written or modified

**Side effects:**
- `.credentials.status` may perform supplementary reads from `~/.claude.json`, `~/.claude/settings.json`, and `{active_name}.json` when corresponding field-presence params are enabled
- No writes, no network requests

### Idempotency

**Yes.** `.credentials.status` is a pure read. Repeated calls return the same result for the same credential state. No side effects accumulate.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `format::` | Output format (`text` or `json`) | No |
| `get::` | Extract a single bare field value for scripting (short-circuits normal rendering) | No |
| `threshold::` | Warning threshold in seconds for `ExpiringSoon` classification | No |
| `trace::` | Emit diagnostic trace output | No |

Field-presence parameters for `.credentials.status` (`account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) each toggle inclusion of the corresponding field in text output. `format::json` always includes all fields regardless.

### State Transition Pattern

**Reads state.** Both commands read `~/.claude/.credentials.json` and optionally supplementary files. No local writes. Credential and account lifecycle state unchanged.

```
[active] --credentials.status-> [active]        (credential metadata + expiry classification read; no change)
```

### See Also

| File | Relationship |
|------|-------------|
| [feature/006_token_status.md](../../feature/006_token_status.md) | Token expiry classification algorithm and threshold semantics |
| [feature/012_live_credentials_status.md](../../feature/012_live_credentials_status.md) | Live credential read algorithm and field sourcing |
| [feature/014_rich_account_metadata.md](../../feature/014_rich_account_metadata.md) | Extended metadata fields from `~/.claude.json` and `{name}.json` |
| [feature/071_redirect_backend_accounts.md](../../feature/071_redirect_backend_accounts.md) | `backend: redirect` accounts classify as `Token: static`/`Expires: N/A` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Show live credential metadata with field presence control, including token expiry classification |
