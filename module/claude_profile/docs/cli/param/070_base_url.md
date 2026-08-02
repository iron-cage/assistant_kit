# Parameter: 70. `base_url::`

The redirect target's API base URL, stored in `{name}.json` and written to `settings.json`'s `env.ANTHROPIC_BASE_URL` whenever this account becomes active.

- **Default:** *(omit; required when `backend::redirect`)*
- **Constraints:** Non-empty string; no format validation beyond non-emptiness — any value is passed through verbatim to `env.ANTHROPIC_BASE_URL`
- **Purpose:** Point the Claude binary at a foreign, Anthropic-API-compatible endpoint instead of `api.anthropic.com`.

**Behavior:** Read only at `.account.save backend::redirect` time and stored in `{name}.json`'s `base_url` field (see [schema/002](../../schema/002_account_json.md)). `.account.use` later reads it back from `{name}.json` and writes it to `settings.json`'s `env.ANTHROPIC_BASE_URL` (see [schema/006](../../schema/006_settings_json.md)) — `base_url::` itself is never accepted by `.account.use`.

**Examples:**

```bash
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-... redirect_model::kimi-k3
```

**Error cases:**
- `backend::redirect` without `base_url::` → exit 1 naming the missing parameter
- `base_url::` present with `backend::anthropic` (or omitted `backend::`) → exit 1 — `base_url::` is redirect-only

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Stores the redirect target base URL |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account |
