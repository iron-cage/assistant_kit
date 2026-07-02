# Commands: Models

Model discovery command.

---

### Command: 19. `.models`

List available Claude API models with capabilities. By default queries the live `GET /v1/models` endpoint using the current account's OAuth token. Use `offline::1` for the static workspace catalog when no credentials are active.

-- **Parameters:** [`format::`](../param/002_format.md), [`offline::`](../param/065_offline.md), [`name::`](../param/001_name.md)
-- **Exit:** 0 (success) | 1 (error: no active token in live mode, API failure)

**Syntax:**

```bash
clp .models                       # live API, table format (default)
clp .models format::json          # json output
clp .models format::text          # one ID per line
clp .models offline::1            # static catalog, no network
clp .models name::claude-opus     # filter by ID substring
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| [`format::`](../param/002_format.md) | [`OutputFormat`](../type/002_output_format.md) | `table` | Output format: text / json / table |
| [`offline::`](../param/065_offline.md) | `bool` | `0` | Use static embedded model catalog; no network call |
| [`name::`](../param/001_name.md) | `string` | *(empty)* | Substring filter on model ID (case-insensitive) |

**Algorithm (live mode, 3 steps):**
1. Obtain OAuth token from current account via `claude_profile_core::account::get_current_token()`.
2. Call `claude_quota::fetch_models(token)` — HTTP GET to `https://api.anthropic.com/v1/models`; collect all pages (limit=1000 per page).
3. Apply `name::` filter if set; render in `format::`.

**Algorithm (offline mode, 2 steps):**
1. Load `STATIC_MODELS` constant from `claude_quota`.
2. Apply `name::` filter if set; render in `format::`.

**Table output columns:**

| Column | Source |
|--------|--------|
| `ID` | `ModelInfo.id` |
| `Display Name` | `ModelInfo.display_name` |
| `Context` | `ModelInfo.max_input_tokens` (formatted as `200k`, `1M`) |
| `Max Out` | `ModelInfo.max_tokens` |
| `Ext Think` | `"extended-thinking"` present in `ModelInfo.capabilities` |

**Examples:**

```bash
clp .models offline::1
# ID                          Display Name       Context  Max Out  Ext Think
# claude-opus-4-8             Claude Opus 4.8    1M       128k     No
# claude-sonnet-5             Claude Sonnet 5    1M       128k     No
# claude-haiku-4-5-20251001   Claude Haiku 4.5   200k     64k      Yes

clp .models offline::1 name::haiku
# claude-haiku-4-5-20251001   Claude Haiku 4.5   200k     64k      Yes

clp .models offline::1 format::json
# [{"id":"claude-opus-4-8","display_name":"Claude Opus 4.8",...}, ...]

clp .models offline::1 format::text
# claude-opus-4-8
# claude-sonnet-5
# claude-haiku-4-5-20251001
```

**Notes:**
- Live mode requires an active account credential. Without credentials, exit 1 with a message suggesting `offline::1`.
- Live mode shows invite-only models accessible to the token-holder's account (e.g., `claude-fable-5` if the account has access). Offline mode shows only the static workspace catalog.
- `name::` on `.models` filters on the model `id` field, not `display_name`.
- `.models` is read-only; use `.model.select id::VALUE` to pin a model for clr subprocesses.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Models List Command](../../feature/068_models_list_command.md) | Full specification for this command |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |
