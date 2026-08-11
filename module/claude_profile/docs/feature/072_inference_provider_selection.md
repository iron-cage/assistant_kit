# Feature: Inference Provider Selection

### Scope

- **Purpose**: Let each saved account be tagged with the inference provider it authenticates against, and let the user select exactly one provider as the active global context — so that automatic rotation never silently mixes accounts across providers (e.g. rotating from an `anthropic` account into a `kimi` account without the user's explicit consent).
- **Responsibility**: Documents the `inference_provider` field in `{name}.json`/`AccountQuota`; the `.provider.select` CLI command and its `~/.clr/config.toml` `provider` backing key; Gate 10's unconditional rotation-eligibility constraint tying the two together; the `inference_provider::` CLI param on `.account.save`; and the default identity-column display of `inference_provider` on `.accounts`.
- **In Scope**: `inference_provider: String` field in `{name}.json`/`Account`/`AccountQuota` (free-form, no allow-list, defaults to `"anthropic"` when absent — never written as the literal default, only ever written when `inference_provider::` is explicitly given); `inference_provider::` param on `.account.save`; `inference_provider` default identity column on `.accounts` (opt-out via `cols::-inference_provider`); the `.provider.select` command (get/set/reset of the `provider` key in `~/.clr/config.toml`'s user tier via `claude_core::toml_io`); Gate 10 in `find_first_eligible()` — unconditional exclusion of accounts whose `inference_provider` does not match the selected `provider`, no `force::1` bypass.
- **Out of Scope**: Any dynamic derivation, fallback chain, or auto-detection of provider — the selected provider is always a single static config scalar, changeable only via an explicit `.provider.select id::` write, never inferred from account state or filtered by request. Any credential/routing mechanism for non-Anthropic endpoints — that is Feature 071's `backend`/`base_url`/`api_key`/`redirect_model` fields, which are independent of and orthogonal to `inference_provider` (a `backend: redirect` account may carry any `inference_provider` label, and a `backend: anthropic` account may too). An allow-list or enum of valid provider names — `inference_provider` and `.provider.select id::` both accept any non-empty string.

### Design

**Why a global scalar, not a filter or fallback chain:** account rotation already has many relative, per-call-site gates (ownership, force bypass, claim lock). Provider selection is deliberately different in kind — it is a single piece of standing user intent ("I am currently working with provider X") that must hold across every rotation decision until the user explicitly changes it. Modeling it as a filter parameter or a derived/fallback value (e.g. "use whichever provider the current account has") would let rotation silently drift across providers as accounts come and go — exactly the failure this feature exists to prevent. So `provider` lives in `~/.clr/config.toml`'s user tier (the same tiered flat-TOML store `.model scope::subprocess` already uses for `model`/`effort`, Feature 035), read once per rotation decision and never derived.

**Why `inference_provider` defaults to `"anthropic"` without being written:** mirrors the existing `backend` field's absent-means-`anthropic` convention (Feature 071) rather than the `host`/`role` metadata labels' write-empty-string convention. Every account created before this feature, and every account saved without `inference_provider::`, has no `inference_provider` key in `{name}.json` at all — readers (`.accounts`/`.usage` rendering, Gate 10) treat that absence as `"anthropic"`. This avoids a one-time migration pass over every existing account file and keeps the common case (single-provider users) free of a redundant explicit tag.

**Why no allow-list:** `inference_provider` is a free-form label, matching the validation-lightness of `host::`/`role::` (also free-form metadata) rather than `backend::` (a closed `AccountBackend` enum). The set of providers a user might tag accounts with is open-ended and not `clp`'s concern to enumerate — `.provider.select id::` accepts the same free-form strings for the same reason. The only validation on either surface is non-empty.

**Why `inference_provider` is a default-shown identity column but has no dedicated toggle parameter:** `.accounts`' existing 32-parameter Help Rendering Scheme (Task 413) is heavily tested against an exact parameter count and group structure. Every other default-shown identity field on `.accounts` (`account`, `owner`, `active`, `current`, `sub`, `tier`, `expires`, `email`) is controlled purely through the shared `cols::` mechanism, not a per-field boolean toggle — `inference_provider` follows that same convention (`cols::-inference_provider` to hide) rather than adding a 33rd parameter.

**Why Gate 10 is unconditional (no `force::1` bypass), mirroring Gate 9:** ownership (Gate 8) and claim-lock (Gate 9) answer different questions than provider selection. Ownership is about *who* may act on an account; `force::1` legitimately overrides it for an authorized caller. Provider selection is about *which provider is currently active* — an absolute categorical property of the rotation context, not a permission. Rotating into a mismatched-provider account would silently switch billing/auth context regardless of who authorized the rotation, so `force::1` (designed to bypass permission checks) must never bypass it. This is the same reasoning `docs/algorithm/004_eligibility_gates.md`'s Gate 9 Context already applies to `claim_lock`.

**Field/responsibility map:**

| Property | Type | Storage | Purpose | Set via | Governs |
|---|---|---|---|---|---|
| `inference_provider` | `String` | `{name}.json` / `Account` / `AccountQuota` | Tags an account with the provider it authenticates against | `.account.save inference_provider::` | `.accounts` default identity column; Gate 10 comparison operand |
| `provider` | `String` (TOML key) | `~/.clr/config.toml` user tier | The single active global provider | `.provider.select id::` | Gate 10 comparison operand; `.provider.select` get-mode read value |

### Acceptance Criteria

- **AC-01**: `clp .account.save name::kimi inference_provider::kimi` exits 0; writes `kimi.json` with `"inference_provider": "kimi"`.
- **AC-02**: `clp .account.save name::alice@acme.com` (no `inference_provider::`) exits 0; `alice@acme.com.json` has no `inference_provider` key at all — not written as `"anthropic"`.
- **AC-03**: `clp .account.save name::kimi inference_provider::` (empty value) exits 1; stderr names `inference_provider::` as requiring a non-empty value; no file written.
- **AC-04**: A pre-existing account file saved before this feature (no `inference_provider` key) is read back by `.accounts`/`.usage`/Gate 10 and treated as `inference_provider: "anthropic"` — no error, no misclassification.
- **AC-05**: `clp .accounts` (no `cols::`) shows a `Provider` column for every account, reading `anthropic` for any account with no `inference_provider` key.
- **AC-06**: `clp .accounts cols::-inference_provider` omits the `Provider` column entirely.
- **AC-07**: `clp .provider.select` (no params, no prior selection) prints `provider.select: anthropic` — never `(unset)`.
- **AC-08**: `clp .provider.select id::kimi` exits 0; writes `provider = "kimi"` into `~/.clr/config.toml`'s user tier; stdout contains `(selected)`.
- **AC-09**: `clp .provider.select id::` (empty value) exits 1; stderr: `id:: must be a non-empty provider name`.
- **AC-10**: `clp .provider.select id::kimi reset::1` (both present) exits 1; stderr: `id:: and reset::1 are mutually exclusive`.
- **AC-11**: `clp .provider.select reset::1` after a prior `id::kimi` selection removes the `provider` key from `~/.clr/config.toml`'s user tier; subsequent `clp .provider.select` prints `provider.select: anthropic`; other keys (e.g. `model`/`effort` written by `.model scope::subprocess`) are preserved unchanged.
- **AC-12**: `clp .provider.select reset::1` with no `~/.clr/config.toml` present exits 0 idempotently — prints `provider.select: anthropic (reset to default)`.
- **AC-13**: `clp .provider.select format::json` prints `{"provider":"anthropic"}` (or the selected value) — JSON key is `provider`, distinct from `.accounts`/`.usage`'s per-row `inference_provider` JSON key.
- **AC-14**: With `provider` selected as `kimi` in `~/.clr/config.toml`, and a mixed account list containing both `inference_provider: "anthropic"` and `inference_provider: "kimi"` accounts, `clp .usage rotate::1` (or auto-rotation) never selects an `anthropic`-tagged account as the next/current target, regardless of `force::1`.
- **AC-15**: With no `provider` ever selected (default `anthropic` in effect), an account with an explicit `inference_provider: "kimi"` tag is never selected by rotation, even though no other gate excludes it — Gate 10 fires using the default `anthropic` comparison value exactly as it would for an explicit selection.
- **AC-16**: `clp .provider.select` never derives its value from any account's `inference_provider` field, current or otherwise — it is a pure read of `~/.clr/config.toml`'s `provider` key, unaffected by which account is currently active.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` gains the `inference_provider` default identity column |
| [002_account_save.md](002_account_save.md) | `.account.save` gains the `inference_provider::` write path |
| [029_account_host_metadata.md](029_account_host_metadata.md) | Sibling free-form metadata-label pattern (`host`/`role`) that `inference_provider::` follows |
| [069_model_select_command.md](069_model_select_command.md) | Superseded — historical `.model.select` design that `.provider.select` originally mirrored |
| [035_model_command.md](035_model_command.md) | `.model scope::subprocess` — current `~/.clr/config.toml`-backed get/set/reset sibling that `.provider.select` mirrors (absorbed `.model.select`'s role) |
| [070_account_claim_and_reservation_control.md](070_account_claim_and_reservation_control.md) | Gate 9 (`claim_lock`, unconditional) — the precedent Gate 10 mirrors |
| [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md) | `backend` field — independent of and orthogonal to `inference_provider` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/073_inference_provider.md](../cli/param/073_inference_provider.md) | `inference_provider::` — tags an account at save time |
| [cli/param/064_id.md](../cli/param/064_id.md) | `id::` — activates set mode on `.provider.select` (narrowed, Feature 035 — formerly also shared with `.model.select`) |
| [cli/param/066_reset.md](../cli/param/066_reset.md) | `reset::` — removes the `provider` key (narrowed, Feature 035 — formerly also shared with `.model.select`) |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.account.save` (`inference_provider::` write path), `.accounts` (default `Provider` identity column) |
| [cli/command/009_provider.md](../cli/command/009_provider.md) | `.provider.select` — full command specification |

### Algorithm Docs

| File | Relationship |
|------|--------------|
| [algorithm/004_eligibility_gates.md](../algorithm/004_eligibility_gates.md) | Gate 10 — unconditional rotation exclusion for `inference_provider` mismatch |

### Schema

| File | Relationship |
|------|--------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | `inference_provider` field in `{name}.json` |
| [../../claude_core/docs/api/002_toml_io.md](../../../claude_core/docs/api/002_toml_io.md) | `~/.clr/config.toml`'s tiered flat-TOML format storing the `provider` key |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account.rs` | `Account` struct — new `inference_provider: String` field (empty/absent-on-omit, mirrors `backend`'s absent-defaults-to-anthropic pattern rather than `host`/`role`'s write-empty-string pattern); `save()` — new read-merge write path for `inference_provider::` |
| `src/usage/types.rs` | `AccountQuota` struct — new `inference_provider: String` field, populated from `{name}.json` at fetch time |
| `src/commands/account_ops.rs` | `account_save_routine()` — new parsing for `inference_provider::`, non-empty validation |
| `src/commands/accounts_render.rs` | New `inference_provider` column rendering for `.accounts` table/json output — default identity set member |
| `src/commands/provider_select.rs` (new) | `.provider.select` command handler — get/set/reset dispatch mirroring `src/commands/model.rs`'s `scope::subprocess` branch (formerly mirrored the now-retired `src/commands/model_select.rs`) |
| `src/usage/sort_next.rs` | `find_first_eligible()` — new Gate 10 check immediately after the existing `claim_lock` check (Gate 9); unconditional, not part of the `extra` closure |
| `src/registry.rs` | New `.provider.select` command registration (Command 21) |
| `src/cli.rs` | New `.provider.select` dispatch wiring |
| `claude_core::toml_io` | Shared `get_tiered`/`set_user_tier`/`remove_user_tier` primitives — reused unchanged from `.model scope::subprocess`'s existing usage (formerly `.model.select`'s) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/provider_select_test.rs` (new) | AC-07–AC-13, AC-16 — get/set/reset behavior, mutual exclusion, JSON format, idempotent reset, no cross-account derivation |
| `tests/cli/accounts_help_test.rs` | Regression coverage — `.accounts.help`'s 32-parameter Help Rendering Scheme unaffected by `inference_provider` (no new toggle param added) |
| `tests/cli/account_save_test.rs` | AC-01–AC-04 — `inference_provider::` write path, empty-value rejection, absent-field default |
| `tests/cli/accounts_render_test.rs` | AC-05–AC-06 — default `Provider` column display and `cols::-inference_provider` opt-out |
| `tests/usage/sort_next_test.rs` | AC-14–AC-15 — Gate 10 exclusion under mixed-provider account lists, with and without an explicit `.provider.select` |
