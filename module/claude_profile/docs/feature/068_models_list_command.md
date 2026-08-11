# Feature: Models List Command

### Scope

- **Purpose**: Provide a `clp .models` command to list available Claude API models with their capabilities, fetched from the live `GET /v1/models` endpoint or from a static embedded catalog.
- **Responsibility**: Documents the `.models` command, its two data sources (live API and static catalog), the `fetch_models()` HTTP function in `claude_quota`, the `format::`, `offline::`, and `name::` parameters, and the `ModelInfo` response type.
- **In Scope**: `.models` command; live mode (default) fetching `GET /v1/models` via `fetch_models()` in `claude_quota` using the current account's OAuth token; offline mode (`offline::1`) returning the static `STATIC_MODELS` constant; `format::table` (default), `format::json`, `format::text`; `name::` substring filter on model ID (case-insensitive); `ModelInfo` deserialization from API response; `STATIC_MODELS` constant embedding the 5-entry catalog from `contract/claude_code/docs/model/readme.md`.
- **Out of Scope**: Model selection for clr subprocesses (→ Feature 035 `scope::subprocess`; formerly Feature 069 `.model.select`, retired); interactive session model writes (→ Feature 035); subprocess model effort control (→ Feature 026); pricing or cloud platform IDs (→ Anthropic docs).

### Design

`.models` operates in live mode by default, using the current account's OAuth token to call the Anthropic `GET /v1/models` endpoint.

**Live mode** (default, `offline::0`):

Obtains the active account's OAuth token from `claude_profile_core::account`, then calls `claude_quota::fetch_models(token)`. The function performs a paginated HTTP fetch from `https://api.anthropic.com/v1/models` with the same headers as existing quota functions (`Authorization: Bearer {token}`, `anthropic-version: 2023-06-01`, `anthropic-beta: oauth-2025-04-20`). Returns the full `data[]` array across all pages (limit=1000 in the first request to avoid multiple round-trips for the current catalog size).

**Offline mode** (`offline::1`):

Returns `STATIC_MODELS` — a `&'static [ModelInfo]` constant embedded in `claude_quota` containing the 5 documented models from the workspace model catalog. No network call is made. Useful when running without an active credential or for scripted comparisons.

**`name::` filter:**

When `name::VALUE` is set, filters the model list to entries where `id.to_ascii_lowercase().contains(&value.to_ascii_lowercase())`. Applied after fetching; returns zero rows (not an error) when no match.

**Output formats:**

`format::table` (default) — human-readable table with columns: `ID | Display Name | Context | Max Out | Ext Think | Status`

`format::text` — one model ID per line.

`format::json` — the raw `data` array serialized as JSON.

**`ModelInfo` type:**

```rust
#[ derive( Debug, Clone, Copy ) ]
pub struct ModelInfo
{
  pub id               : &'static str,
  pub display_name     : &'static str,
  pub created_at       : Option< &'static str >,
  pub max_input_tokens : Option< u64 >,
  pub max_tokens       : Option< u64 >,
  pub capabilities     : &'static [ &'static str ],
}
```

All string-bearing fields are `&'static str` (not owned `String`/`Vec`) so that `STATIC_MODELS` can be a `const` and the type can derive `Copy`; entries returned by live-mode `fetch_models()` are heap-allocated and leaked once per process to satisfy the same `&'static` signature — acceptable for a short-lived CLI process.

The `capabilities` field is used to derive the `Ext Think` column in table format: present if `"extended-thinking"` appears in the capabilities list.

**Static catalog** (`STATIC_MODELS` constant in `claude_quota/src/lib.rs`):

Derived from `contract/claude_code/docs/model/readme.md`. Contains 5 entries, all with `created_at: None` (static entries carry no creation timestamp) and conservative static context/max-output values:

| ID | Display Name | Max Input | Max Output | Capabilities |
|----|---------------|-----------|------------|--------------|
| `claude-opus-4-8` | Claude Opus 4.8 | 200,000 | 32,000 | `extended-thinking` |
| `claude-sonnet-5` | Claude Sonnet 5 | 200,000 | 64,000 | — |
| `claude-haiku-4-5-20251001` | Claude Haiku 4.5 | 200,000 | 32,000 | — |
| `claude-opus-4-5-20251101` | Claude Opus 4.5 | 200,000 | 32,000 | `extended-thinking` |
| `claude-sonnet-4-5-20250929` | Claude Sonnet 4.5 | 200,000 | 64,000 | — |

### Acceptance Criteria

- **AC-01**: `clp .models offline::1` — stdout contains `claude-opus-4-8`. Exits 0.
- **AC-02**: `clp .models offline::1` — stdout contains `claude-sonnet-5`. Exits 0.
- **AC-03**: `clp .models offline::1` — stdout contains `claude-haiku-4-5-20251001`. Exits 0.
- **AC-04**: `clp .models offline::1 format::table` — output has header row with `ID` column. Exits 0.
- **AC-05**: `clp .models offline::1 format::text` — output is one ID per line; no table formatting. Exits 0.
- **AC-06**: `clp .models offline::1 format::json` — output is valid JSON array `[{...}]`. Exits 0.
- **AC-07**: `clp .models offline::1 name::opus` — output contains only models with `opus` in the ID; all other models absent. Exits 0.
- **AC-08**: `clp .models offline::1 name::zz_no_match` — output is empty (zero rows); exits 0.
- **AC-09**: `clp .models` is listed in `clp .help` output.
- **AC-10**: `clp .models offline::1 name::claude-opus` — returns all models with `claude-opus` in the ID; filter is substring, case-insensitive.

### Features

| File | Relationship |
|------|--------------|
| [035_model_command.md](035_model_command.md) | `.model` command — interactive session model get/set |
| [069_model_select_command.md](069_model_select_command.md) | Superseded — historical `.model.select` design; subprocess model IDs discovered here are now set via `.model scope::subprocess model::` (Feature 035) |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` — subprocess model for touch/refresh (separate mechanism) |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/models.rs` | `.models` command handler |
| `src/registry.rs` | Registration of `.models` command and parameters |
| `claude_quota/src/lib.rs` | `fetch_models()` HTTP function; `STATIC_MODELS` constant; `ModelInfo` type |
| `claude_profile_core/src/account.rs` | OAuth token retrieval for live mode |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/068_models_list_command.md` | FT-01 through FT-10 |
| `tests/docs/cli/command/19_models.md` | IT-01 through IT-10 |
