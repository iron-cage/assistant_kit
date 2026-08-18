# Verb: save

Captures the current active session credentials as a named account profile in the credential store. Creates `{name}.credentials.json` and `{name}.json` under the per-machine store path, performing a read-merge on any pre-existing supplementary metadata.

### Nouns

| # | Noun | Command | Idempotent | Requires Session |
|---|------|---------|-----------|-----------------|
| 1 | [account](../command_noun/001_account.md) | `.account.save` | Conditional | Yes |

### Behavioral Contract

**Pre-conditions:**
- Active Claude Code session exists (`~/.claude/.credentials.json` readable and valid)
- `$HOME` environment variable set

**Post-conditions:**
- Named account profile exists in credential store (`{name}.credentials.json` + `{name}.json`)
- `{name}.json` supplementary fields merged from any pre-existing file (existing fields preserved)
- Active session credentials unchanged
- When `backend::redirect`: `{name}.credentials.json` contains only `accessToken` (from `api_key::`) — no `refreshToken`/`expiresAt` keys; `{name}.json` carries `backend`, `base_url`, `redirect_model`

**Side effects:**
- Writes two files per account: `{name}.credentials.json` (credential snapshot) and `{name}.json` (supplementary metadata)
- If `name::` resolves to an existing profile, existing `{name}.json` fields are preserved via read-merge (not overwritten)
- When `backend::redirect`: does not read `~/.claude/.credentials.json` — no active OAuth session is captured for a foreign backend

### Idempotency

**Conditional.** Re-running `.account.save name::EMAIL` with the same credentials produces the same stored state. Running again after a credential change overwrites the credential snapshot but preserves unchanged supplementary metadata via read-merge. Not idempotent across credential changes.

### Common Parameters

| Parameter | Semantics | Required |
|-----------|-----------|----------|
| `name::` | Account name (email); defaults to email from active credentials | No |
| `host::` | Override store host label for multi-machine profiles | No |
| `role::` | Annotate account with a role label (e.g. `work`, `personal`) — REMOVED when Feature 075 lands, superseded by `tags::` | No |
| `tags::` 📋 | Replace account's tag set (comma-separated; planned — Feature 075) | No |
| `backend::` | Discriminate `anthropic` (default, OAuth capture) vs `redirect` (static API key) | No |
| `base_url::` | Redirect target's API base URL | Conditional (`backend::redirect`) |
| `api_key::` | Redirect target's static API key, written as `accessToken` | Conditional (`backend::redirect`) |
| `redirect_model::` | Redirect target's own model identifier | Conditional (`backend::redirect`) |
| `dry::` | Validate without writing files | No |
| `trace::` | Emit diagnostic trace output | No |

### State Transition Pattern

**Creates state.** Writes `{name}.credentials.json` (credential copy) and `{name}.json` (supplementary metadata) into the per-machine credential store. If the profile already exists, transitions its credential snapshot to the current session state.

```
[absent]  --account.save--> [saved]
[saved]   --account.save--> [saved]  (credential snapshot updated; {name}.json read-merged)
[active]  --account.save--> [active] (active account re-saved; no lifecycle change)
```

### See Also

| File | Relationship |
|------|-------------|
| [feature/001_account_store_init.md](../../feature/001_account_store_init.md) | Credential store initialization before first save |
| [feature/002_account_save.md](../../feature/002_account_save.md) | Save algorithm and `{name}.json` read-merge semantics |
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | Ownership model — `.account.save` is ownership-neutral (passes `owner: None`); `.accounts owner::0 name::X` clears ownership (Feature 064); `.accounts assignee::USER@MACHINE name::X` is marker-only (Feature 065) |
| [feature/071_redirect_backend_accounts.md](../../feature/071_redirect_backend_accounts.md) | `backend::redirect` write path — static-credential capture via `base_url::`/`api_key::`/`redirect_model::`, no OAuth |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Capture current credentials as named profile |
