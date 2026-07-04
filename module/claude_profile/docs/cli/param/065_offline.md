# Parameter: 65. `offline::`

Selects the static embedded model catalog instead of the live API for `.models`. When `offline::1`, no network call is made and no OAuth token is required.

- **Default:** `0` — live mode (queries `GET /v1/models` with OAuth token)
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** List models without network access; useful in scripts, CI, or environments without active credentials.

**Values:**

| Value | Effect |
|-------|--------|
| `0` (default) | Live mode: fetch from `GET /v1/models` using current account OAuth token |
| `1` | Offline mode: return `STATIC_MODELS` constant embedded in `claude_quota`; no network call |

**Error cases:**
- `offline::0` (default) without valid credentials → exit 1; stderr suggests using `offline::1`

**Examples:**

```bash
clp .models offline::1            # static catalog, no network
clp .models offline::1 format::json
clp .models offline::1 name::haiku
```

**Notes:**
- Offline mode returns the workspace-curated catalog from `claude_quota::STATIC_MODELS`, derived from `contract/claude_code/docs/model/readme.md`. This catalog may lag behind the live API when new models are released.
- Live mode is preferred when checking invite-only model access (e.g., `claude-fable-5`) or confirming the latest available models.
- Combine with `name::` to quickly look up a specific model without network access: `clp .models offline::1 name::claude-opus-4-8`.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

*(ungrouped — `.models`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.models`](../command/008_models.md) | Selects data source: live API (`0`) or static catalog (`1`) |
