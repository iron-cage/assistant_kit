# Type: 5. `AccountBackend`

**Purpose:** Selects whether a saved account authenticates against Anthropic's own OAuth-backed API or a foreign, Anthropic-API-compatible endpoint (e.g. a Moonshot Kimi K3 proxy) reached via a static API key. Determines which fields are read/written in `{name}.json` and `{name}.credentials.json`, and which code paths `.account.save`/`.account.use` take.

**Fundamental Type:** Enum — two named variants

**Constants:**
- `ANTHROPIC` — the existing OAuth-backed flow: credentials captured from `~/.claude/.credentials.json`, subject to expiry/refresh, session model governed by `apply_model_override()`
- `REDIRECT` — a foreign endpoint reached via `base_url::`/`api_key::`/`redirect_model::`; static, non-expiring credential; `apply_model_override()` bypassed
- `DEFAULT = Anthropic`

**Constraints:**
- One of: `anthropic`, `redirect` (case-insensitive)
- Unknown values rejected with exit 1
- Fixed at `.account.save` creation time — no command changes an existing account's `backend` after creation (recreate via `.account.save` again to change it)

**Parsing:**

```
pub fn new( s : &str ) -> Result< Self, String >
```

**Methods:**
- `get() -> &str` — string representation (`"anthropic"` or `"redirect"`)
- `is_redirect() -> bool` — true for the redirect backend

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`backend::`](../param/069_backend.md) | Selects the backend at `.account.save` time |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Creates the account with the given backend |
| 2 | [`.account.use`](../command/001_account.md#command-5-accountuse) | Branches switch behavior on the target account's stored `backend` |
| 3 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Reports the active account's `backend`; classifies `Static` when `backend == "redirect"` |
| 4 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | Rejects/skips for `backend == "redirect"` — no Anthropic quota to query |
| 5 | [`.account.inspect`](../command/001_account.md#command-15-accountinspect) | Rejects/skips for `backend == "redirect"` — no Anthropic identity/quota to query |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Saving a foreign-backend account alongside Anthropic accounts |
