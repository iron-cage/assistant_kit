# Parameter: 72. `redirect_model::`

The foreign provider's own model identifier, stored in `{name}.json` and written to `settings.json`'s `env.ANTHROPIC_MODEL` whenever this account becomes active.

- **Default:** *(omit; required when `backend::redirect`)*
- **Constraints:** Non-empty string; no format validation — this is the foreign backend's own model catalog, unrelated to `clp`'s Anthropic model shorthands (`opus`/`sonnet`/`haiku`)
- **Purpose:** Tell the redirect target which of its models to serve (e.g. a Moonshot Kimi model ID).

**Behavior:** Read only at `.account.save backend::redirect` time and stored in `{name}.json`'s `redirect_model` field (see [schema/002](../../schema/002_account_json.md)). `.account.use` later reads it back and writes it to `settings.json`'s `env.ANTHROPIC_MODEL` (see [schema/006](../../schema/006_settings_json.md)). Distinct from `set_model::`/`imodel::`, which select among Anthropic's own `opus`/`sonnet`/`haiku` shorthands and never apply to a redirect account (see [algorithm/002](../../algorithm/002_session_model_override.md)'s redirect bypass).

**Examples:**

```bash
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-... redirect_model::kimi-k3-0905-preview
```

**Error cases:**
- `backend::redirect` without `redirect_model::` → exit 1 naming the missing parameter
- `redirect_model::` present with `backend::anthropic` (or omitted `backend::`) → exit 1 — `redirect_model::` is redirect-only

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Stores the redirect target model identifier |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account |
