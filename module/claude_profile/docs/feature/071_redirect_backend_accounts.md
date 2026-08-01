# Feature: Redirect Backend Accounts

### Scope

- **Purpose**: Let `.account.save` create accounts that point Claude Code at a foreign, Anthropic-API-compatible endpoint (e.g. Moonshot Kimi K3) via a static API key, instead of capturing an Anthropic OAuth session — while leaving every existing Anthropic-backend account and workflow completely unaffected.
- **Responsibility**: Documents the `backend`/`base_url`/`redirect_model` fields in `{name}.json`; the `accessToken`-only `{name}.credentials.json` shape for redirect accounts; the new `static` state_machine state; the `backend::`/`base_url::`/`api_key::`/`redirect_model::` CLI params and the `AccountBackend` type; `apply_model_override()`'s redirect bypass; `.account.use`'s new responsibility to write/clear `settings.json`'s `env.*` keys and to skip quota-fetch/touch entirely for redirect targets; the Anthropic-only operation guard on `.account.limits`/`.account.inspect`; the `backend` column surfaced by `.accounts` and the `Token: static` classification that signals backend on `.credentials.status`; and the per-row quota-fetch skip for redirect accounts in `.usage`/`.accounts`'s shared fetch pipeline.
- **In Scope**: `backend: AccountBackend` (enum, default `anthropic`, fixed per save call), `base_url: Option<String>`, `redirect_model: Option<String>` fields in `{name}.json`; `accessToken`-only `{name}.credentials.json` for redirect accounts (no `refreshToken`/`expiresAt` keys — genuinely absent, not null); `backend::`, `base_url::`, `api_key::`, `redirect_model::` params, currently scoped to `.account.save`; `AccountBackend` enum type; new `static` state_machine state (terminal-stable, checked before any expiry threshold math); `TokenStatus::Static` classification; `apply_model_override()` redirect no-op bypass; `.account.use` writing `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL` into `settings.json` on switch-to-redirect and removing exactly those three keys on switch-to-anthropic; `pre_switch_touch_ctx()`'s unconditional skip of quota-fetch/touch for redirect targets; `.account.limits`/`.account.inspect`'s Anthropic-only guard; `backend` column addition to `.accounts` and `Token: static` signaling on `.credentials.status`; a redirect-bypass gate in `fetch_quota_for_list()` (shared by `.usage`/`.accounts`) producing a per-row `Err`-result placeholder with no HTTP call.
- **Out of Scope**: Any provisioning flow for foreign API keys — a raw `api_key::` string is the only supported input, no OAuth-like exchange for redirect backends. Wire-format translation or proxying between Anthropic's API shape and a foreign provider's own — `base_url` must already be Anthropic-API-compatible; this is a redirect, not an adapter. Usage/quota telemetry for redirect accounts — no Anthropic quota endpoint exists for a foreign backend, so `.usage`/`.account.limits` cannot report real utilization (see Anthropic-only guard, which rejects rather than approximates). Changing `backend` on an existing account in place — `backend` is fixed per save call; switching requires re-running `.account.save` with the same `name::`, which rewrites the account from scratch rather than partially updating it.

### Design

**Why not model this as a differently-configured Anthropic account:** existing accounts assume Anthropic OAuth end-to-end — a `refreshToken`, an `expiresAt`, a live quota/usage API. A foreign endpoint has none of these: no refresh flow, no expiry on `clp`'s clock, no Anthropic quota data to report. Rather than force these absent concepts into optional/nullable fields on the existing shape, redirect accounts are modeled as a distinct `backend` discriminator with their own minimal field set — `base_url`/`redirect_model` are meaningless for `backend: anthropic`, and `refreshToken`/`expiresAt` are meaningless for `backend: redirect`.

**How Claude Code actually picks up the foreign endpoint:** `clp` never sets process environment variables directly — those would not persist to a later, independently-launched `claude` process. Instead, `.account.use` writes an `env` object into `~/.claude/settings.json` (`env.ANTHROPIC_BASE_URL`, `env.ANTHROPIC_AUTH_TOKEN`, `env.ANTHROPIC_MODEL`), which the Claude binary reads natively at its own startup — the same mechanism a user would otherwise configure by hand.

**No settings.json formatter changes needed:** `claude_core::settings_io` already provides `set_env_var()`/`remove_env_var()` (built for the pre-existing `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES` auto-updater toggles), which read-modify-write a single key inside the nested `env` sub-object while preserving every other `env` sub-key and every other top-level `settings.json` field — `json_serialize_flat_object`'s `infer_type()` already classifies any value starting with `{`/`[` as `StoredAs::Raw` and emits it verbatim, so a pre-serialized nested object round-trips through the "flat" formatter unchanged (see [schema/006](../schema/006_settings_json.md)). `switch_account()` reuses these existing, already-tested primitives directly — three `set_env_var()`/`remove_env_var()` calls per switch direction — with zero new code in `claude_core`. The one genuinely new behavior is dropping the outer `env` key entirely when the switch-to-anthropic direction removes its last remaining sub-key (AC-07) — `remove_env_var()` alone leaves `"env": {}` behind, so `switch_account()` (in `claude_profile_core`) additionally checks post-removal emptiness via the already-public `get_setting()`/`remove_setting()` and prunes `env` itself when empty; this composition lives entirely in `claude_profile_core`, not in the shared formatter.

**Field/responsibility map:**

| Property | Type | Storage | Purpose | Set via | Governs |
|---|---|---|---|---|---|
| `backend` | `AccountBackend` (enum) | `{name}.json` | Discriminates `anthropic` (OAuth) vs `redirect` (foreign endpoint) accounts | `.account.save backend::` | Write-path branch in `.account.save`; bypass in `apply_model_override()`; guard in `.account.limits`/`.account.inspect`; `static` state entry in `refresh_account_token()`/`.credentials.status` |
| `base_url` | `Option<String>` | `{name}.json` | Foreign endpoint's API base URL | `.account.save base_url::` | `.account.use` → `settings.json`'s `env.ANTHROPIC_BASE_URL` |
| `redirect_model` | `Option<String>` | `{name}.json` | Foreign endpoint's own model identifier | `.account.save redirect_model::` | `.account.use` → `settings.json`'s `env.ANTHROPIC_MODEL` |
| `accessToken` (redirect) | `string` | `{name}.credentials.json` | Static API key, stored verbatim | `.account.save api_key::` | `.account.use` → `settings.json`'s `env.ANTHROPIC_AUTH_TOKEN` |
| `refreshToken`/`expiresAt` (redirect) | *(absent)* | `{name}.credentials.json` | N/A — no OAuth session for a foreign backend | n/a | `static` state entry: absence of `expiresAt` is the trigger |

**`static` state — checked first, no transitions:** `refresh_account_token()`, `.credentials.status`, and `.account.use`'s expiry probe all check `backend == redirect` (equivalently, `expiresAt` absence) before any threshold math — a redirect account is always classified `static`, never `valid`/`expiring_soon`/`expired`. `static` is terminal-stable: no transition moves an account into or out of it except re-running `.account.save`, which rewrites the account rather than transitioning its state.

**`apply_model_override()` redirect bypass:** checks `backend` first; when `backend == redirect`, returns as a no-op, writing neither `model` nor `effortLevel`. The Sonnet-vs-Opus quota tradeoff this function otherwise resolves has no meaning for a foreign backend, whose own model is fixed by `redirect_model` and written directly to `env.ANTHROPIC_MODEL` on switch.

**Anthropic-only operation guard:** `.account.limits` and `.account.inspect` reject (non-zero exit, explanatory stderr) when invoked against a `backend: redirect` account — both query live Anthropic-only endpoints (rate-limit headers, org/identity data) that do not exist for a foreign provider. No approximation or partial output is attempted; the guard is a hard rejection.

**`.account.save`'s redirect write path:** when `backend::redirect`, `.account.save` does NOT copy `~/.claude/.credentials.json` — there is no active OAuth session to capture for a foreign backend. Instead it writes `{name}.credentials.json` containing only `accessToken` (from `api_key::`), and stores `base_url`/`redirect_model` alongside `backend` in `{name}.json`.

**`.account.use`'s redirect write/clear responsibility:** switching TO a `backend: redirect` account writes all three `env.*` sub-keys into `settings.json` from that account's `base_url`/`accessToken`/`redirect_model`. Switching TO a `backend: anthropic` account removes exactly those three sub-keys (removing `env` entirely if it becomes empty as a result), while preserving any other unrelated `env` sub-key and every other unrelated `settings.json` top-level field.

**`.account.use`'s quota-fetch/touch skip for redirect targets:** `pre_switch_touch_ctx()` checks the target account's `backend` before attempting any quota-fetch or touch subprocess step (Feature 027); when `backend == redirect`, the entire step is skipped unconditionally — no HTTP call is attempted and no failure is recorded, since no Anthropic quota endpoint exists to touch. This differs from the existing quota-fetch-fails-then-skips path: here there is no attempt at all, not a failed one.

**`fetch_quota_for_list()`'s redirect bypass (`.usage`/`.accounts`):** the shared multi-account quota-fetch pipeline (Feature 037) checks each account's `backend` before its existing non-owned gate; a `backend: redirect` account produces an `AccountQuota` row with `result: Err("redirect backend — no Anthropic quota")` and no HTTP call. No further pipeline change is required: `apply_touch()`'s pre-existing error-account guard already skips touching any `Err`-result row, and the existing failed-fetch rendering path already displays 🔴 status with `—` quota columns and a reason string — both for free.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::kimi backend::redirect base_url::https://api.moonshot.ai/anthropic api_key::sk-test redirect_model::kimi-k3-0905-preview` exits 0; writes `kimi.json` with `backend: "redirect"`, `base_url`, `redirect_model`; writes `kimi.credentials.json` containing only `accessToken: "sk-test"` — no `refreshToken`/`expiresAt` keys present at all.
- **AC-02**: The same command missing any one of `base_url::`/`api_key::`/`redirect_model::` exits 1, stderr names the specific missing parameter(s); no files written.
- **AC-03**: `clp .account.save name::alice@acme.com base_url::https://x` (no `backend::redirect`) exits 1 — `base_url::` is redirect-only and rejected outside that context.
- **AC-04**: `clp .account.save name::alice@acme.com` (no `backend::` at all) preserves existing behavior exactly — copies `~/.claude/.credentials.json`; writes `alice@acme.com.json` with `backend: "anthropic"`.
- **AC-05**: A pre-existing account file saved before Feature 071 (no `backend` key at all) is read back and treated as `backend: anthropic` — `.accounts`/`.credentials.status`/etc. neither error nor misclassify it.
- **AC-06**: `clp .account.use name::kimi` writes `env.ANTHROPIC_BASE_URL`, `env.ANTHROPIC_AUTH_TOKEN`, `env.ANTHROPIC_MODEL` into `settings.json` matching `kimi.json`/`kimi.credentials.json`'s stored values; pre-existing unrelated `settings.json` top-level fields (e.g. `model`, `effortLevel` from a prior anthropic account) are left untouched.
- **AC-07**: `clp .account.use name::alice@acme.com` (switching to an anthropic account) after a prior redirect switch removes exactly `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL` from `settings.json`; `env` itself is removed if it becomes empty as a result; any other unrelated `env.*` sub-key present before the switch is preserved unchanged.
- **AC-08**: Superseded by AC-14 — this criterion originally targeted the now-removed `.token.status` command; `.credentials.status`'s `Token: static` classification (AC-14) is the single surviving criterion for this behavior.
- **AC-09**: `refresh_account_token()` invoked against a `backend: redirect` account is a no-op — no refresh subprocess spawned, no credential write-back.
- **AC-10**: `apply_model_override()` invoked while the active account is `backend: redirect` writes neither `model` nor `effortLevel` to `settings.json` — confirmed byte-for-byte unchanged aside from the unrelated `env.*` keys AC-06 already covers.
- **AC-11**: `clp .account.limits name::kimi` exits non-zero with a message naming the operation as Anthropic-only; no HTTP request is made.
- **AC-12**: `clp .account.inspect name::kimi` exits non-zero with the same Anthropic-only guard message; no HTTP request is made.
- **AC-13**: `clp .accounts cols::+backend` shows a `backend` column with `anthropic`/`redirect` per account; `format::json` always includes the `backend` field regardless of `cols::` (existing json-always-includes-all-fields rule).
- **AC-14**: `clp .credentials.status name::kimi` reports `Token: static` (never `valid`/`expiring_soon`/`expired`) for the active redirect account — no separate `backend` field is needed on this single-account command, since the `Token:` line's classification already signals it; `refreshToken`/`expiresAt`-derived fields report absent/N/A rather than erroring.
- **AC-15**: Re-running `clp .account.save name::kimi backend::anthropic` (same name, different backend) rewrites `kimi.json`/`kimi.credentials.json` from scratch per the anthropic path (captures current `~/.claude/.credentials.json`) — a single save call cannot mix fields from both backends, but the account name itself is not permanently locked to one backend.
- **AC-16**: `clp .account.use name::kimi` (target `backend: redirect`, default `touch::1`) skips quota-fetch and the touch subprocess entirely — exits 0 with zero HTTP calls attempted; behaviorally distinct from `touch::0` (a param-driven skip) since this skip is backend-driven and unconditional.
- **AC-17**: `clp .usage` / `clp .accounts refresh::1` with a `backend: redirect` account present in the account list renders that account's row with `—` for quota columns and a shortened reason string (e.g. `redirect backend — no Anthropic quota`), with no HTTP call made for that row; other `backend: anthropic` rows in the same listing are unaffected.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | `.account.save` gains the `backend::redirect` write path — bypasses OAuth credential capture |
| [004_account_use.md](004_account_use.md) | `.account.use` gains the `env.*` write/clear responsibility in `settings.json` |
| [006_token_status.md](006_token_status.md) | Token classification gains the new `static` value alongside Valid/ExpiringSoon/Expired |
| [011_account_status_by_name.md](011_account_status_by_name.md) | `.credentials.status` (by name) surfaces the `static` classification for redirect accounts |
| [012_live_credentials_status.md](012_live_credentials_status.md) | `.credentials.status` gains the `backend` field and graceful absent-field handling |
| [013_account_limits.md](013_account_limits.md) | `.account.limits` gains the Anthropic-only guard, rejecting redirect accounts |
| [017_token_refresh.md](017_token_refresh.md) | `refresh_account_token()` gains the redirect no-op bypass, checked before threshold math |
| [024_session_touch.md](024_session_touch.md) | Touch subprocess is entirely skipped for redirect targets — no idle-window activation to attempt |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | `pre_switch_touch_ctx()` gains the redirect backend check — skips quota-fetch/subprocess unconditionally |
| [031_account_inspect.md](031_account_inspect.md) | `.account.inspect` gains the same Anthropic-only guard as `.account.limits` |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `apply_model_override()` gains the redirect no-op bypass |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/069_backend.md](../cli/param/069_backend.md) | `backend::` — discriminates `anthropic`/`redirect` at save time |
| [cli/param/070_base_url.md](../cli/param/070_base_url.md) | `base_url::` — redirect target's API base URL |
| [cli/param/071_api_key.md](../cli/param/071_api_key.md) | `api_key::` — redirect target's static API key |
| [cli/param/072_redirect_model.md](../cli/param/072_redirect_model.md) | `redirect_model::` — redirect target's own model identifier |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — target account identifier for `.account.save` |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` (redirect write path), `.account.use` (env.* write/clear), `.account.limits` (Anthropic-only guard), `.account.inspect` (Anthropic-only guard) |
| [cli/command/002_credentials.md](../cli/command/002_credentials.md) | `.credentials.status` — new `backend` field; new `static` classification |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage`/`.accounts`' shared quota-fetch pipeline — per-row redirect bypass, no HTTP call, `—` quota columns |

### Algorithm Docs

| File | Relationship |
|------|--------------|
| [algorithm/002_session_model_override.md](../algorithm/002_session_model_override.md) | Redirect bypass — `apply_model_override()` no-op when `backend == redirect` |

### Schema

| File | Relationship |
|------|--------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | `backend`, `base_url`, `redirect_model` fields in `{name}.json` |
| [schema/001_credentials_json.md](../schema/001_credentials_json.md) | `accessToken`-only shape for redirect accounts; absent `refreshToken`/`expiresAt` |
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `env.ANTHROPIC_BASE_URL`/`env.ANTHROPIC_AUTH_TOKEN`/`env.ANTHROPIC_MODEL`; nested-object formatter exception |

### State Machines

| File | Relationship |
|------|--------------|
| [state_machine/002_oauth_token_lifecycle.md](../state_machine/002_oauth_token_lifecycle.md) | `static` state — terminal-stable, no transitions to/from Anthropic states |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account.rs` | `Account` struct — new `backend: AccountBackend`, `base_url: Option<String>`, `redirect_model: Option<String>` fields; new `AccountBackend` enum; `save()` (existing 8-param fn) — new redirect-specific write path bypassing `~/.claude/.credentials.json` capture; `switch_account()` — new responsibility to write/clear `settings.json`'s `env.*` keys via `claude_core::settings_io`'s existing `set_env_var()`/`remove_env_var()`, plus new empty-`env`-pruning logic (AC-07) composed from the existing `get_setting()`/`remove_setting()`; `refresh_account_token()` — new redirect no-op bypass, checked before any refresh-threshold math (AC-09) |
| `claude_profile_core/src/token.rs` | `TokenStatus` enum — new `Static` variant |
| `claude_profile/src/commands/account_ops.rs` | `account_save_routine()` — new parsing/validation for `backend::`/`base_url::`/`api_key::`/`redirect_model::` and dispatch to the redirect write path; `account_use_routine()` — call site for `switch_account()`'s new `env.*` responsibility |
| `src/usage/api_switch.rs` | `apply_model_override()` — new `backend` check, returns as a no-op when `backend == redirect`; `pre_switch_touch_ctx()` — new `backend` check, skips quota-fetch/touch unconditionally when `backend == redirect` |
| `src/usage/fetch.rs` | `fetch_quota_for_list()` — new redirect-bypass gate, checked before the existing non-owned gate; produces an `Err`-result placeholder row with no HTTP call |
| `src/commands/limits.rs` | `account_limits_routine()` — new Anthropic-only guard rejecting `backend: redirect` accounts |
| `src/commands/account_inspect.rs` | `account_inspect_routine()` — new Anthropic-only guard rejecting `backend: redirect` accounts |
| `src/commands/credentials.rs` | `credentials_status_routine()` — new `backend` field in output; graceful absence handling for `refreshToken`/`expiresAt`-derived fields; new `Static` classification branch, checked before existing threshold math |
| `src/commands/accounts_render.rs` | New `backend` column rendering for `.accounts` table/json output |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_redirect_backend_test.rs` | AC-01–AC-07, AC-09–AC-17 — redirect save (happy path + missing-param errors), anthropic-path non-regression, pre-existing-account default, env.* write on switch-to-redirect, env.* clear on switch-to-anthropic (with unrelated-key preservation), static classification, refresh no-op, model-override no-op, limits/inspect guards, accounts/credentials.status backend field, re-save backend change, touch/quota-fetch skip on switch, usage/accounts redirect-row placeholder |
