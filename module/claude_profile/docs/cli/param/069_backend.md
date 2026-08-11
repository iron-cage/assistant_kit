# Parameter: 69. `backend::`

Selects which backend a newly-saved account authenticates against: Anthropic's own OAuth-backed API, or a foreign Anthropic-API-compatible endpoint reached via a static API key.

- **Default:** `anthropic`
- **Constraints:** `anthropic`, `redirect` (case-insensitive)
- **Purpose:** Let `.account.save` create accounts that redirect Claude Code to a non-Anthropic model provider (e.g. Moonshot Kimi K3) instead of capturing the current OAuth session.

**Values:**

| Value | Effect |
|-------|--------|
| `anthropic` (default) | Existing behavior — capture `~/.claude/.credentials.json` into the new account |
| `redirect` | Requires `base_url::`, `api_key::`, `redirect_model::`; writes a static-credential account with no OAuth fields |

**Behavior:** `backend::redirect` switches `.account.save` to a different write path (see [feature/071](../../feature/071_redirect_backend_accounts.md)): instead of copying `~/.claude/.credentials.json`, it writes `{name}.credentials.json` containing only `accessToken` (from `api_key::`), and stores `base_url`/`redirect_model` in `{name}.json`. `backend` is fixed at creation — re-run `.account.save` with the same `name::` to change it (this rewrites the account from scratch, it is not a partial update).

**Examples:**

```bash
clp .account.save name::alice@acme.com                                          # backend defaults to anthropic
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-... redirect_model::kimi-k3
```

**Error cases:**
- `backend::bad` → exit 1 with stderr naming valid values: `anthropic`, `redirect`
- `backend::redirect` without one of `base_url::`/`api_key::`/`redirect_model::` → exit 1 naming the missing parameter(s)

**Notes:**
- Pre-existing accounts (saved before Feature 071) have no `backend` field on disk; they are treated as `anthropic` (see [schema/002](../../schema/002_account_json.md)'s Preserved-Only Fields).
- `backend::anthropic` explicitly is accepted but has no effect beyond the existing default behavior — provided for symmetry/scriptability, not required in normal use.

### Referenced Type

- **Fundamental Type:** [`AccountBackend`](../type/005_account_backend.md) (`enum`)

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | Member parameter — discriminator |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Selects the backend for the new account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account |
