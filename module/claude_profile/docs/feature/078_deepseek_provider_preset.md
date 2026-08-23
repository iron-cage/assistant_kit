# Feature: DeepSeek Provider Preset

### Scope

- **Purpose**: Make adding a DeepSeek redirect-backend account a four-parameter operation (`name::`, `preset::deepseek`, `api_key::`, `redirect_model::`) instead of a seven-parameter one, and give `switch_account()` the tier-substitution/effort/compact-window environment contract DeepSeek's own Claude Code integration guide documents — the second bespoke preset alongside Feature 073's Kimi, added the same deliberate way (see [type/007](../type/007_preset.md)'s "each new preset is an explicit design addition, not a data entry").
- **Responsibility**: Documents the `preset::deepseek` value on `.account.save` (second recognized value alongside `kimi`, same convenience/gating/precedence mechanics as Feature 073); the 6 additional `settings.json` `env.*` variables `switch_account()` writes for a `backend: redirect` account whose `inference_provider` is exactly `"deepseek"`; the fixed (non-branching) compact-window value; and the cross-provider clearing behavior that keeps a stale Kimi-tier or DeepSeek-tier bundle from surviving a switch between the two.
- **In Scope**: `deepseek` as a second recognized `preset::` value (validated, matched case-insensitively, same precedence/gating rules as `kimi`); the 6 DeepSeek-tier `settings.json` env vars (`ANTHROPIC_DEFAULT_OPUS_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL` — mirror `redirect_model`; `ANTHROPIC_DEFAULT_HAIKU_MODEL`, `CLAUDE_CODE_SUBAGENT_MODEL` — fixed `"deepseek-v4-flash"`; `CLAUDE_CODE_EFFORT_LEVEL` — fixed `"max"`; `CLAUDE_CODE_AUTO_COMPACT_WINDOW` — fixed `"786432"`) written/cleared by `switch_account()`; the cross-provider clear (switching directly between a Kimi-tagged and a DeepSeek-tagged redirect account clears the bundle that no longer applies).
- **Out of Scope**: Any provider other than Kimi and DeepSeek — `preset::` recognizes exactly these two values; each additional provider remains its own reviewed design addition, never a config-driven registry entry (see [type/007](../type/007_preset.md)). Validating that `api_key::`/`redirect_model::` are genuine DeepSeek credentials — `clp` never calls out to a provider at save time (see [feature/071](071_redirect_backend_accounts.md)'s Out of Scope). Any change to the 3 base redirect env vars or Feature 073's Kimi-tier bundle — both remain exactly as previously defined; this feature only adds a second, independently-gated 6-variable bundle alongside them. DeepSeek API feature parity/limitations versus the real Anthropic Messages API (unsupported content types, disabled parallel tool use, etc., per DeepSeek's own docs) — those are provider-side behaviors `clp` neither controls nor validates.

### Design

**Why a second bespoke preset, not a generalized provider registry:** [type/007_preset.md](../type/007_preset.md) and Feature 073's own Out-of-Scope section both state the single-value constraint was "a deliberate scope boundary, not a placeholder for a future provider registry." Adding DeepSeek the same way Kimi was added — a reviewed feature doc plus explicit code branches — preserves that decision rather than reversing it. A third provider, if one is ever needed, gets the same treatment; two data points is not yet grounds to extract a table (Rule of Three), and a hand-authored per-provider behavioral bundle is the only way to faithfully encode each provider's own, differently-shaped integration contract (see next point).

**Why DeepSeek's env-var bundle is shaped differently from Kimi's, not just re-parameterized:** verified directly against DeepSeek's own official "Integrate with Claude Code" guide (`api-docs.deepseek.com/quick_start/agent_integrations/claude_code`), fetched live rather than assumed — the two providers' documented contracts are genuinely different shapes, not the same shape with different strings:
  - **4 tier vars, not 5 — no Fable var.** Kimi's `KIMI_MODEL_TIER_ENV_VARS` includes `ANTHROPIC_DEFAULT_FABLE_MODEL`; DeepSeek's guide does not document one. Inventing a Fable mapping DeepSeek never specified would be writing an uncertain detail as certain. Leaving it unset is safe: DeepSeek's Anthropic-API-compatibility docs state unrecognized model names fall back to `deepseek-v4-flash` server-side, so a Fable-tier request is still bounded, just not pinned by `clp`.
  - **Two distinct values, not one uniform mirror.** Kimi's preset mirrors a single `redirect_model` value into all 5 tier vars. DeepSeek's guide instead documents a **Pro/Flash split**: `ANTHROPIC_DEFAULT_OPUS_MODEL`/`ANTHROPIC_DEFAULT_SONNET_MODEL` mirror the account's own `redirect_model` (the heavier model the caller saved the account against, e.g. `deepseek-v4-pro`), while `ANTHROPIC_DEFAULT_HAIKU_MODEL`/`CLAUDE_CODE_SUBAGENT_MODEL` always get a fixed lighter model (`deepseek-v4-flash`), regardless of `redirect_model`. This mirrors how Kimi's own compact-window sizing is a fixed function of a model check rather than a caller-supplied value — DeepSeek's flash substitution is the analogous "provider knows its own cost/speed tradeoff better than a per-account parameter should" case, just applied to model selection instead of window sizing.
  - **Flat compact-window, no model-name branching.** Kimi's `kimi_auto_compact_window()` branches on whether `redirect_model` starts with `kimi-k3` (1M window) or not (256K). DeepSeek's guide gives one number, `786432`, with no documented per-model variance — consistent with DeepSeek V4-Pro and V4-Flash publicly sharing the same 1M-token context window (no smaller-context tier exists to protect against with a narrower default). Adding a branch with no second value to branch on would be speculative, not evidence-based.

**Why the Flash-tier value is a fixed literal, never a caller-supplied parameter:** the two per-account values Feature 071/073 established (`api_key::`, `redirect_model::`) remain exactly that — `redirect_model::` names the Pro-tier model the account is saved against. Adding a second CLI parameter for the Flash-tier model would expand Feature 071's schema (a new persisted field) for a value that is, per DeepSeek's own guide, structurally the same for every DeepSeek account (its cheapest/fastest current model) — exactly the kind of provider-constant fact `preset::` exists to collapse, not re-expose as a parameter. If DeepSeek's recommended Flash model changes, updating `DEEPSEEK_FLASH_MODEL` in one place (`switch.rs`) updates it for every DeepSeek account, the same way `kimi_auto_compact_window()`'s literal thresholds are a single edit point today.

**Why switching between a Kimi-tagged and a DeepSeek-tagged redirect account must explicitly clear the other's bundle:** Feature 073 only ever had one provider-tier bundle to manage — every non-Kimi redirect switch cleared it unconditionally, and there was no second bundle that could still be live. With two bundles, a switch from a Kimi account directly to a DeepSeek account (or vice versa) must clear the bundle belonging to the account being switched *away from*, not just skip writing the one for the account being switched *to* — otherwise the previous provider's 5 or 6 stale vars survive alongside the new provider's own, corrupting Claude Code's tier resolution with two conflicting sets of overrides. `patch_live_state_after_switch()` therefore always calls both `write_*_tier_env_vars()` for the matching provider (if any) **and** `clear_*_tier_env_vars()` for the non-matching one(s) — see [schema/006](../schema/006_settings_json.md) for the full write/clear table.

**Precedence and gating summary (mirrors Feature 073's table exactly, second value only):**

| Field | Preset default (when `preset::deepseek` AND resolved `backend == redirect`) | Overridable by |
|---|---|---|
| `backend` | `redirect` (applied only when `backend::` itself was omitted) | Explicit `backend::` |
| `base_url` | `https://api.deepseek.com/anthropic` | Explicit `base_url::` |
| `inference_provider` | `deepseek` | Explicit `inference_provider::` |
| `api_key` | *(never defaulted — always explicit)* | n/a |
| `redirect_model` | *(never defaulted — always explicit)* | n/a |

**Field/responsibility map (env vars written by `switch_account()` for `inference_provider: "deepseek"`):**

| Property | Type | Storage | Purpose | Set via | Governs |
|---|---|---|---|---|---|
| `ANTHROPIC_DEFAULT_OPUS_MODEL` / `ANTHROPIC_DEFAULT_SONNET_MODEL` | `string` (env var) | `settings.json` `env.*` | Substitutes the account's own `redirect_model` (Pro-tier) for the Opus/Sonnet Anthropic tier names | `switch_account()` (both mirror `redirect_model` verbatim) | Claude binary's Opus/Sonnet-tier model resolution |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` / `CLAUDE_CODE_SUBAGENT_MODEL` | `string` (env var, always `"deepseek-v4-flash"`) | `settings.json` `env.*` | Routes Haiku-tier and subagent dispatch to DeepSeek's lighter/faster model, independent of `redirect_model` | `switch_account()` (both fixed to `DEEPSEEK_FLASH_MODEL`) | Claude binary's Haiku-tier and subagent model resolution |
| `CLAUDE_CODE_EFFORT_LEVEL` | `string` (env var, always `"max"`) | `settings.json` `env.*` | Forces maximum reasoning effort for the DeepSeek redirect target | `switch_account()` | Claude binary's thinking/effort depth |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `string` (env var, always `"786432"`) | `settings.json` `env.*` | Context-window token count at which Claude Code auto-compacts | `switch_account()` — fixed, no model-name branching | Claude binary's auto-compaction threshold |

### Acceptance Criteria

- **AC-01**: `switch_account()` to a `backend: redirect` account with `inference_provider: "deepseek"` writes all 3 Feature-071 base vars plus all 6 DeepSeek-tier vars into `settings.json`'s `env` object; `ANTHROPIC_DEFAULT_OPUS_MODEL`/`ANTHROPIC_DEFAULT_SONNET_MODEL` equal the account's `redirect_model` value; `ANTHROPIC_DEFAULT_HAIKU_MODEL`/`CLAUDE_CODE_SUBAGENT_MODEL` equal `"deepseek-v4-flash"`; `CLAUDE_CODE_EFFORT_LEVEL` equals `"max"`.
- **AC-02**: `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is written as `"786432"` regardless of the `redirect_model` value (e.g. both `deepseek-v4-pro` and `deepseek-v4-flash` as the saved `redirect_model` produce the same window) — no model-name branching, unlike Kimi's Feature 073 AC-02.
- **AC-03**: `switch_account()` to a `backend: redirect` account whose `inference_provider` is absent or is any value other than exactly `"deepseek"` (including `"kimi"`) writes only the 3 Feature-071 base vars — none of the 6 DeepSeek-tier vars appear in `settings.json`.
- **AC-04**: `switch_account()` from an active `inference_provider: "deepseek"` redirect account to a `backend: anthropic` account removes all 9 vars (the 3 base vars per Feature 071, plus all 6 DeepSeek-tier vars) rather than leaving the 6 DeepSeek-tier vars stale.
- **AC-05**: `switch_account()` from an active `inference_provider: "deepseek"` redirect account to a *different* `backend: redirect` account whose `inference_provider` is not `"deepseek"` writes that account's own 3 base vars and removes the 6 now-stale DeepSeek-tier vars.
- **AC-06**: `clp .account.save name::deepseek preset::deepseek api_key::sk-test redirect_model::deepseek-v4-pro` (no explicit `backend::`/`base_url::`/`inference_provider::`) exits 0; `deepseek.json` is written with `backend: "redirect"`, `base_url: "https://api.deepseek.com/anthropic"`, `inference_provider: "deepseek"`.
- **AC-07**: The same command with an explicit `base_url::https://custom.endpoint/anthropic` added exits 0 and `deepseek.json` stores that explicit value — `preset::deepseek`'s own default value is never applied when the caller supplies `base_url::` directly.
- **AC-08**: `clp .account.save name::alice@acme.com preset::deepseek backend::anthropic` exits 0 via the ordinary `backend::anthropic` OAuth-capture path — `preset::deepseek`'s `base_url`/`inference_provider` defaults are never applied, because the resolved `backend` is `anthropic`, not `redirect`.
- **AC-09**: `clp .account.save name::x preset::bogus api_key::sk-test redirect_model::m1` exits 1; stderr names both recognized values (`kimi`, `deepseek`); no files are written.
- **AC-10**: `clp .account.save name::deepseek preset::deepseek api_key::sk-test redirect_model::deepseek-v4-pro` followed by `clp .account.use name::deepseek` writes all 9 env vars into `settings.json` end-to-end through the CLI surface, confirming AC-01's domain-layer behavior is reachable from `.account.save`/`.account.use` together.
- **AC-11**: Switching directly from an active `inference_provider: "kimi"` redirect account to an `inference_provider: "deepseek"` redirect account clears all 7 Kimi-tier vars and writes all 6 DeepSeek-tier vars (no Kimi-tier var survives alongside the new DeepSeek-tier bundle); switching in the reverse direction (`deepseek` → `kimi`) equally clears all 6 DeepSeek-tier vars and writes all 7 Kimi-tier vars.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md) | Base 3-variable `env.*` write/clear mechanism this feature adds 6 more variables alongside, keyed off the same `backend: redirect` switch |
| [072_inference_provider_selection.md](072_inference_provider_selection.md) | `inference_provider` field — the exact-match `"deepseek"` gate that determines whether the 6 DeepSeek-tier vars are written |
| [073_kimi_provider_preset.md](073_kimi_provider_preset.md) | Sibling preset — same `preset::` mechanism and gating doctrine, independently-shaped behavioral bundle; AC-11 covers the cross-provider clear between the two |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/074_preset.md](../cli/param/074_preset.md) | `preset::` — second recognized value (`deepseek`), pre-fills `backend::`/`base_url::`/`inference_provider::` |
| [cli/param/069_backend.md](../cli/param/069_backend.md) | `backend::` — resolved value gates whether `preset::deepseek`'s redirect-only defaults apply |
| [cli/param/070_base_url.md](../cli/param/070_base_url.md) | `base_url::` — defaulted by `preset::deepseek` when the resolved backend is `redirect` and no explicit value was given |
| [cli/param/073_inference_provider.md](../cli/param/073_inference_provider.md) | `inference_provider::` — defaulted by `preset::deepseek` under the same condition |
| [cli/param/072_redirect_model.md](../cli/param/072_redirect_model.md) | `redirect_model::` — never defaulted; its value is mirrored into the 2 Pro-tier env vars only (Flash-tier vars use a fixed literal instead) |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` (`preset::deepseek` convenience value), `.account.use` (`switch_account()`'s DeepSeek-tier env var write/clear) |

### Schema

| File | Relationship |
|------|--------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | 6 new `env.*` DeepSeek-tier fields; design section detailing the write/clear rule and cross-provider clearing |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account/switch.rs` | New `DEEPSEEK_PRO_TIER_ENV_VARS`/`DEEPSEEK_FLASH_TIER_ENV_VARS` consts, `DEEPSEEK_FLASH_MODEL`/`DEEPSEEK_AUTO_COMPACT_WINDOW` consts, `write_deepseek_tier_env_vars()`/`clear_deepseek_tier_env_vars()` helpers; `patch_live_state_after_switch()` — 3-way branch on `inference_provider` (`kimi` / `deepseek` / other) that writes the matching bundle and clears the non-matching one(s), including the anthropic-switch cleanup path |
| `claude_profile/src/commands/account_ops.rs` | `account_save_routine()` — `preset::` parsing extended to accept `deepseek` alongside `kimi`; default-filling for `backend`/`base_url`/`inference_provider` extended with DeepSeek's own values |
| `claude_profile/src/registry.rs` | `preset::` param help text updated to list both recognized values |

### Tests

| File | Relationship |
|------|--------------|
| `claude_profile_core/tests/account_backend_test.rs` | AC-01–AC-05, AC-11 — DeepSeek-tier env var writing on switch, flat compact-window regardless of model, non-deepseek-provider omission, clearing on switch-to-anthropic and switch-to-other-redirect, cross-provider clear in both directions between Kimi and DeepSeek |
| `claude_profile/tests/cli/account_redirect_backend_test.rs` | AC-06–AC-10 — `preset::deepseek` default-filling, explicit-value override, end-to-end env var write via `.account.save` + `.account.use`, `preset::deepseek` + explicit `backend::anthropic` non-interference; AC-09 — existing unrecognized-preset test extended to assert both recognized values are named |
