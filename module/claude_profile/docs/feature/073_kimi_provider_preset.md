# Feature: Kimi Provider Preset

### Scope

- **Purpose**: Make adding a Moonshot Kimi redirect-backend account a four-parameter operation (`name::`, `preset::kimi`, `api_key::`, `redirect_model::`) instead of a seven-parameter one, and give `switch_account()` the full 10-variable environment contract Kimi's own Claude Code integration guide documents — not just the 3 generic redirect variables Feature 071 covers.
- **Responsibility**: Documents the `preset::` CLI param on `.account.save` (convenience default-filler for `backend::`/`base_url::`/`inference_provider::`, gated on the resolved `backend` rather than the bare preset flag); the 7 additional `settings.json` `env.*` variables `switch_account()` writes for a `backend: redirect` account whose `inference_provider` is exactly `"kimi"`; and the compact-window sizing rule that distinguishes Kimi K3 models from any other redirect model sharing the same `inference_provider` tag.
- **In Scope**: `preset::` param on `.account.save` (only recognized value: `kimi`); its precedence rule (explicit `backend::`/`base_url::`/`inference_provider::` always override the preset's defaults) and its gating rule (the redirect-only defaults apply only when the *resolved* `backend` is `redirect`, never merely because `preset::kimi` was given); the 7 Kimi-tier `settings.json` env vars (`ANTHROPIC_DEFAULT_OPUS_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_HAIKU_MODEL`, `ANTHROPIC_DEFAULT_FABLE_MODEL`, `CLAUDE_CODE_SUBAGENT_MODEL`, `CLAUDE_CODE_EFFORT_LEVEL`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`) written/cleared by `switch_account()` alongside Feature 071's existing 3; the `kimi-k3*`-vs-other compact-window sizing rule.
- **Out of Scope**: Any provider other than Kimi — `preset::` recognizes exactly one value today; this is a deliberate scope boundary, not a placeholder for a future provider registry. Validating that `api_key::`/`redirect_model::` are genuine Moonshot credentials — `clp` never calls out to a provider at save time (see [feature/071](071_redirect_backend_accounts.md)'s Out of Scope). Any change to the 3 base redirect env vars (`ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`/`ANTHROPIC_MODEL`) or their write/clear mechanics — those remain exactly as Feature 071 defined them; this feature only adds 7 more variables alongside them, gated on a different condition (`inference_provider == "kimi"` rather than `backend == redirect`).

### Design

**Why a preset, not a new required-params tuple:** the 10-variable environment contract Kimi's own documentation specifies is entirely mechanical — `base_url` is always the same fixed Moonshot endpoint, `inference_provider` is always `"kimi"`, and both derive from a single fact ("this account talks to Kimi"). Requiring the caller to spell out `backend::redirect base_url::https://api.moonshot.ai/anthropic inference_provider::kimi` by hand on every Kimi account creation duplicates that fact three times and invites a mismatched typo in `base_url` going unnoticed. `preset::kimi` collapses it to a single flag while leaving the two genuinely per-account values (`api_key::`, `redirect_model::`) always explicit.

**Why `preset::`, not `provider::`:** `inference_provider` (Feature 072, `.provider.select`) already owns the vocabulary "provider" for a different purpose — tagging an account for Gate 10 rotation grouping. Naming this parameter `provider::` would create a Term Collision between two parameters that mean different things (one selects convenience defaults at save time, the other tags an account for rotation-eligibility comparison at any time). `preset::` names what the parameter actually is: a named bundle of default values, analogous to a config preset, with no relationship to Gate 10 or `.provider.select` beyond incidentally filling the same `inference_provider` field one of them reads.

**Why defaults are gated on resolved `backend`, not on `preset::kimi` being present:** a caller may combine `preset::kimi` with an explicit `backend::anthropic` — perhaps scripting a loop over provider presets where only some rows are meant to be redirect accounts. If the gate checked bare `preset_is_kimi` instead of the resolved `backend` value, `base_url::`/`inference_provider::` would be silently force-filled onto what the caller explicitly declared an `anthropic`-backend save, corrupting an otherwise-valid anthropic account with meaningless redirect metadata. Gating on the already-resolved `backend` variable (itself computed from `preset::kimi`'s own `backend::redirect` default when `backend::` was omitted) makes `preset::kimi backend::anthropic` behave exactly as if `preset::` had never been given.

**Why `api_key::`/`redirect_model::` are never defaulted by the preset:** these are the two genuinely per-account values — an API key is a secret unique to each account, and `redirect_model::` varies by which Kimi model tier or subscription the caller has. Defaulting either would mean either hardcoding a credential (a security defect) or guessing a model the caller may not have access to. `preset::kimi` only fills values that are structurally identical for every Kimi account, never values that vary per account.

**Why 7 more env vars instead of reusing the existing 3:** Feature 071's 3 variables (`ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`/`ANTHROPIC_MODEL`) are sufficient to route traffic to any Anthropic-API-compatible foreign endpoint, but they say nothing about *how* Claude Code should behave once connected — which model ID to substitute for each of Anthropic's own named tiers (Opus/Sonnet/Haiku/Fable) when a subagent or slash-command hardcodes one of those names, how much reasoning effort to request, or when to auto-compact context. Kimi's own Claude Code integration guide documents 10 total `settings.json` env vars for exactly this reason — the 3 routing vars plus 7 behavioral ones. This feature adds the 7 behavioral vars, scoped narrowly to accounts tagged `inference_provider: "kimi"` (Feature 072's field) so that a redirect account for some *other* foreign provider — sharing Feature 071's `backend: redirect` mechanism but not Kimi's specific behavioral contract — never receives Kimi-specific tuning it has no use for.

**Why the compact-window value depends on `redirect_model` matching `kimi-k3*`:** Kimi K3-tier models support a substantially larger context window than earlier Kimi tiers. Sizing `CLAUDE_CODE_AUTO_COMPACT_WINDOW` too large for a smaller-context model risks a real context-overflow failure (the model rejects or truncates the request); sizing it too small for a K3 model only costs a minor, safe degradation (compaction runs somewhat more often than strictly necessary). Given that asymmetry, the model-name check defaults to the smaller, safer value (`"262144"`) for anything that isn't recognizably `kimi-k3*`, and only widens to `"1048576"` when the model name itself confirms K3-tier capacity.

**Precedence and gating summary:**

| Field | Preset default (when `preset::kimi` AND resolved `backend == redirect`) | Overridable by |
|---|---|---|
| `backend` | `redirect` (applied only when `backend::` itself was omitted) | Explicit `backend::` |
| `base_url` | `https://api.moonshot.ai/anthropic` | Explicit `base_url::` |
| `inference_provider` | `kimi` | Explicit `inference_provider::` |
| `api_key` | *(never defaulted — always explicit)* | n/a |
| `redirect_model` | *(never defaulted — always explicit)* | n/a |

**Field/responsibility map (env vars written by `switch_account()`):**

| Property | Type | Storage | Purpose | Set via | Governs |
|---|---|---|---|---|---|
| `preset` | *(not persisted — `.account.save`-only convenience sugar)* | n/a | Named default bundle for `backend::`/`base_url::`/`inference_provider::` | `.account.save preset::` | Pre-fills the 3 fields above at save time; has no runtime existence after save completes |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` / `ANTHROPIC_DEFAULT_SONNET_MODEL` / `ANTHROPIC_DEFAULT_HAIKU_MODEL` / `ANTHROPIC_DEFAULT_FABLE_MODEL` / `CLAUDE_CODE_SUBAGENT_MODEL` | `string` (env var) | `settings.json` `env.*` | Substitutes the account's own `redirect_model` for every Anthropic tier name and for subagent dispatch | `switch_account()` (all 5 mirror `redirect_model` verbatim) | Claude binary's per-tier and per-subagent model resolution |
| `CLAUDE_CODE_EFFORT_LEVEL` | `string` (env var, always `"max"`) | `settings.json` `env.*` | Forces maximum reasoning effort for the Kimi redirect target | `switch_account()` | Claude binary's thinking/effort depth |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `string` (env var) | `settings.json` `env.*` | Context-window token count at which Claude Code auto-compacts | `switch_account()` — `"1048576"` when `redirect_model` starts with `kimi-k3`, else `"262144"` | Claude binary's auto-compaction threshold |

### Acceptance Criteria

- **AC-01**: `switch_account()` to a `backend: redirect` account with `inference_provider: "kimi"` writes all 3 Feature-071 base vars plus all 7 Kimi-tier vars into `settings.json`'s `env` object; the 5 tier/subagent model vars each equal the account's `redirect_model` value, and `CLAUDE_CODE_EFFORT_LEVEL` equals `"max"`.
- **AC-02**: `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is written as `"1048576"` when `redirect_model` starts with `kimi-k3`, and as `"262144"` for any other `redirect_model` value.
- **AC-03**: `switch_account()` to a `backend: redirect` account whose `inference_provider` is absent or is any value other than exactly `"kimi"` writes only the 3 Feature-071 base vars — none of the 7 Kimi-tier vars appear in `settings.json`.
- **AC-04**: `switch_account()` from an active `inference_provider: "kimi"` redirect account to a `backend: anthropic` account removes all 10 vars (the 3 base vars per Feature 071, plus all 7 Kimi-tier vars) rather than leaving the 7 Kimi-tier vars stale.
- **AC-05**: `switch_account()` from an active `inference_provider: "kimi"` redirect account to a *different* `backend: redirect` account whose `inference_provider` is not `"kimi"` writes that account's own 3 base vars and removes the 7 now-stale Kimi-tier vars.
- **AC-06**: `clp .account.save name::kimi preset::kimi api_key::sk-test redirect_model::kimi-k3` (no explicit `backend::`/`base_url::`/`inference_provider::`) exits 0; `kimi.json` is written with `backend: "redirect"`, `base_url: "https://api.moonshot.ai/anthropic"`, `inference_provider: "kimi"`.
- **AC-07**: The same command with an explicit `base_url::https://custom.endpoint/anthropic` added exits 0 and `kimi.json` stores that explicit value — `preset::kimi`'s own default value is never applied when the caller supplies `base_url::` directly.
- **AC-08**: `clp .account.save name::alice@acme.com preset::kimi backend::anthropic` exits 0 via the ordinary `backend::anthropic` OAuth-capture path (see [feature/071](071_redirect_backend_accounts.md)) — `preset::kimi`'s `base_url`/`inference_provider` defaults are never applied, because the resolved `backend` is `anthropic`, not `redirect`.
- **AC-09**: `clp .account.save name::x preset::bogus api_key::sk-test redirect_model::m1` exits 1; stderr names the only recognized value (`kimi`); no files are written.
- **AC-10**: `clp .account.save name::kimi preset::kimi api_key::sk-test redirect_model::kimi-k3` followed by `clp .account.use name::kimi` writes all 10 env vars into `settings.json` end-to-end through the CLI surface, confirming AC-01's domain-layer behavior is reachable from `.account.save`/`.account.use` together.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md) | Base 3-variable `env.*` write/clear mechanism this feature adds 7 more variables alongside, keyed off the same `backend: redirect` switch |
| [072_inference_provider_selection.md](072_inference_provider_selection.md) | `inference_provider` field — the exact-match `"kimi"` gate that determines whether the 7 Kimi-tier vars are written |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/074_preset.md](../cli/param/074_preset.md) | `preset::` — pre-fills `backend::`/`base_url::`/`inference_provider::` for a known provider |
| [cli/param/069_backend.md](../cli/param/069_backend.md) | `backend::` — resolved value gates whether `preset::kimi`'s redirect-only defaults apply |
| [cli/param/070_base_url.md](../cli/param/070_base_url.md) | `base_url::` — defaulted by `preset::kimi` when the resolved backend is `redirect` and no explicit value was given |
| [cli/param/073_inference_provider.md](../cli/param/073_inference_provider.md) | `inference_provider::` — defaulted by `preset::kimi` under the same condition |
| [cli/param/072_redirect_model.md](../cli/param/072_redirect_model.md) | `redirect_model::` — never defaulted; its value is mirrored into the 5 tier/subagent model env vars |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` (`preset::` convenience param), `.account.use` (`switch_account()`'s Kimi-tier env var write/clear) |

### Schema

| File | Relationship |
|------|--------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | 7 new `env.*` Kimi-tier fields; design section detailing the write/clear rule and compact-window sizing |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account/switch.rs` | New `KIMI_MODEL_TIER_ENV_VARS` const, `kimi_auto_compact_window()`, `write_kimi_tier_env_vars()`, `clear_kimi_tier_env_vars()` helpers; `patch_live_state_after_switch()` — new branch checking `inference_provider == "kimi"` on a redirect switch, invoking these helpers instead of (alongside) the plain 3-var write, and clearing them on switch-to-anthropic and switch-to-non-kimi-redirect |
| `claude_profile/src/commands/account_ops.rs` | `account_save_routine()` — new `preset::` parsing, validation (only `kimi` recognized), and default-filling for `backend`/`base_url`/`inference_provider`, gated on the resolved `backend` |
| `claude_profile/src/registry.rs` | New `preset::` param registration on `.account.save` |

### Tests

| File | Relationship |
|------|--------------|
| `claude_profile_core/tests/account_test.rs` | AC-01–AC-05 — Kimi-tier env var writing on switch, `kimi-k3*`-vs-other compact window sizing, non-kimi-provider omission, clearing on switch-to-anthropic and switch-to-other-redirect |
| `claude_profile/tests/cli/account_redirect_backend_test.rs` | AC-06–AC-10 — `preset::kimi` default-filling, explicit-value override, unrecognized preset rejection, end-to-end env var write via `.account.save` + `.account.use`, `preset::kimi` + explicit `backend::anthropic` non-interference |
