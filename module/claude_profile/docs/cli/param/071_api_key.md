# Parameter: 71. `api_key::`

The static API key (credential) for a redirect-backend account, written verbatim as `accessToken` in `{name}.credentials.json`.

- **Default:** *(omit; required when `backend::redirect`)*
- **Constraints:** Non-empty string; no format validation — passed through verbatim
- **Purpose:** Supply the foreign provider's credential directly, since there is no local OAuth session to capture for a non-Anthropic backend.

**Behavior:** Read only at `.account.save backend::redirect` time. Written as the sole field of `{name}.credentials.json` — no `refreshToken`, no `expiresAt` (see [schema/001](../../schema/001_credentials_json.md)'s Redirect Backend Example). Never written to `~/.claude/.credentials.json` at save time — only `.account.use` propagates it, into `settings.json`'s `env.ANTHROPIC_AUTH_TOKEN` (see [schema/006](../../schema/006_settings_json.md)).

**Examples:**

```bash
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-moonshot-abc123 redirect_model::kimi-k3-0905-preview
```

**Error cases:**
- `backend::redirect` without `api_key::` → exit 1 naming the missing parameter
- `api_key::` present with `backend::anthropic` (or omitted `backend::`) → exit 1 — `api_key::` is redirect-only

**Notes:**
- Unlike `.account.save`'s normal (anthropic) flow, which never accepts a credential value on the command line (it copies `~/.claude/.credentials.json`), `api_key::` is the one case where a raw credential is passed as a CLI argument — callers should be mindful of shell history when invoking this directly (prefer environment variable expansion, e.g. `api_key::"$KIMI_API_KEY"`, over a literal key in an interactive shell).

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Stores the redirect target credential |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account |
