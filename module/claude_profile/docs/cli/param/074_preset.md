# Parameter: 74. `preset::`

Named provider preset that pre-fills `backend::`/`base_url::`/`inference_provider::` for a known foreign provider, so creating a redirect-backend account needs only the account-specific values (`name::`, `api_key::`, `redirect_model::`).

- **Default:** *(omit; no preset applied — every field must be given explicitly)*
- **Constraints:** `kimi` or `deepseek` (case-insensitive) are the two recognized values; any other non-empty value is a usage error
- **Purpose:** Collapse the fixed, provider-constant parts of adding a known-provider account (`backend::redirect`, that provider's base URL, its `inference_provider::` tag) into a single flag, while leaving account-specific values (`api_key::`, `redirect_model::`) always explicit.

**Values:**

| Value | Effect |
|-------|--------|
| *(omit, default)* | No defaults applied — `backend::`/`base_url::`/`inference_provider::` behave exactly as they would with no `preset::` present |
| `kimi` | Fills `backend::redirect` when `backend::` was omitted; fills `base_url::https://api.moonshot.ai/anthropic` and `inference_provider::kimi` when those were omitted AND the resolved `backend` is `redirect` |
| `deepseek` | Fills `backend::redirect` when `backend::` was omitted; fills `base_url::https://api.deepseek.com/anthropic` and `inference_provider::deepseek` when those were omitted AND the resolved `backend` is `redirect` |

**Behavior:** Explicit parameters always win over the preset's defaults — passing `base_url::` or `inference_provider::` directly stores exactly that value, never the preset's. The `base_url`/`inference_provider` defaults are gated on the *resolved* `backend` value (after the preset's own `backend` default is applied), not on `preset::` being present in isolation — so `preset::kimi backend::anthropic` (or `preset::deepseek backend::anthropic`) never force-fills redirect-only fields onto an anthropic-backend save; the account saves via the ordinary OAuth-capture path exactly as if `preset::` had not been given. `preset::` never defaults `api_key::` or `redirect_model::` — these remain always-explicit, per-account values (see [feature/073](../../feature/073_kimi_provider_preset.md), [feature/078](../../feature/078_deepseek_provider_preset.md)).

**Examples:**

```bash
clp .account.save name::kimi preset::kimi api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
# equivalent to: backend::redirect base_url::https://api.moonshot.ai/anthropic inference_provider::kimi

clp .account.save name::kimi preset::kimi base_url::https://custom.endpoint/anthropic api_key::"$KIMI_API_KEY" redirect_model::kimi-k3
# explicit base_url:: overrides the kimi preset's default endpoint

clp .account.save name::alice@acme.com preset::kimi backend::anthropic
# preset::kimi's redirect-only defaults never apply — backend stays anthropic, ordinary OAuth capture runs

clp .account.save name::deepseek preset::deepseek api_key::"$DEEPSEEK_API_KEY" redirect_model::deepseek-v4-pro
# equivalent to: backend::redirect base_url::https://api.deepseek.com/anthropic inference_provider::deepseek
```

**Error cases:**
- `preset::bogus` → exit 1 with stderr naming the two recognized values: `kimi`, `deepseek`

**Notes:**
- `preset::kimi` also drives `switch_account()`'s Kimi-tier `settings.json` env var writes indirectly, by filling `inference_provider::kimi` — see [feature/073](../../feature/073_kimi_provider_preset.md) and [schema/006](../../schema/006_settings_json.md). `preset::deepseek` does the same for the DeepSeek-tier env vars via `inference_provider::deepseek` — see [feature/078](../../feature/078_deepseek_provider_preset.md).
- Scoped to exactly two recognized values today — `kimi` and `deepseek` — by design; not a general multi-provider registry (see [feature/073](../../feature/073_kimi_provider_preset.md)'s Out of Scope). Each additional provider is its own reviewed design addition, not a data-driven registry entry.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Redirect Backend Config](../param_group/007_redirect_backend_config.md) | Member parameter — convenience default-filler for the other four |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Pre-fills `backend::`/`base_url::`/`inference_provider::` for a known provider |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Kimi Provider Preset](../../feature/073_kimi_provider_preset.md) | Full feature specification for `kimi` |
| 2 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | `backend::`/`base_url::` fields this preset fills |
| 3 | [Inference Provider Selection](../../feature/072_inference_provider_selection.md) | `inference_provider::` field this preset fills |
| 4 | [DeepSeek Provider Preset](../../feature/078_deepseek_provider_preset.md) | Full feature specification for `deepseek` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | Adding a Kimi-compatible account with the minimal required parameters |
