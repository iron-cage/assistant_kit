# Group: 7. Redirect Backend Config

**Parameters:** `backend::`, `base_url::`, `api_key::`, `redirect_model::`
**Pattern:** Foreign-backend account creation fields — required together, rejected outside their scope
**Purpose:** Let `.account.save` create an account that redirects Claude Code to a non-Anthropic, Anthropic-API-compatible endpoint instead of capturing the current OAuth session.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`backend::`](../param/069_backend.md) | [`AccountBackend`](../type/005_account_backend.md) (`enum`) | `anthropic` | Selects `anthropic` (existing OAuth flow) or `redirect` (foreign endpoint) |
| [`base_url::`](../param/070_base_url.md) | `string` | *(omit; required when `backend::redirect`)* | Redirect target's API base URL |
| [`api_key::`](../param/071_api_key.md) | `string` | *(omit; required when `backend::redirect`)* | Redirect target's static API key |
| [`redirect_model::`](../param/072_redirect_model.md) | `string` | *(omit; required when `backend::redirect`)* | Redirect target's own model identifier |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | All 4 params — creates a redirect-backend account |

**Typical Patterns:**

```bash
# Create a redirect-backend account
clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::"$KIMI_API_KEY" redirect_model::kimi-k3-0905-preview

# Anthropic accounts are unaffected — backend:: defaults to anthropic
clp .account.save name::alice@acme.com
```

**Membership rule:** `base_url::`, `api_key::`, `redirect_model::` are accepted only alongside `backend::redirect` — any of the three present with `backend::anthropic` (or `backend::` omitted) exits 1. Conversely, `backend::redirect` without all three present exits 1 naming the missing parameter(s). `backend::` alone (defaulting to `anthropic`, or explicitly `backend::anthropic`) never requires the other three.

**Semantic Coherence Test**

> "Does parameter X only make sense in the context of creating or describing a redirect-backend account?"

`backend::` passes: it is the discriminator the other three depend on. `base_url::`, `api_key::`, `redirect_model::` each pass: none has any meaning for an `anthropic` account. This is the inverse membership rule from [Account Targeting](006_account_targeting.md), whose Semantic Coherence Test explicitly excludes authentication data — these four parameters are exactly that authentication/backend-selection data.

**Cross-References**

- [../../feature/071_redirect_backend_accounts.md](../../feature/071_redirect_backend_accounts.md) — feature spec for redirect-backend accounts
- [../../schema/002_account_json.md](../../schema/002_account_json.md) — `backend`, `base_url`, `redirect_model` fields in `{name}.json`
- [../../schema/001_credentials_json.md](../../schema/001_credentials_json.md) — `accessToken`-only credential shape for redirect accounts
- [../type/005_account_backend.md](../type/005_account_backend.md) — `AccountBackend` type specification

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account alongside Anthropic accounts |
