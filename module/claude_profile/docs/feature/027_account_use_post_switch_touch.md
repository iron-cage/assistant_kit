# Feature: `.account.use` Post-Switch Touch

### Scope

- **Purpose**: Activate the switched-to account's idle 5h session window immediately after credential rotation, using the same model/effort resolution as `.usage`; expose the post-switch touch lifecycle via `trace::` for diagnostics.
- **Responsibility**: Documents the `touch::`, `imodel::`, `effort::`, and `trace::` parameters on `.account.use` and the post-switch quota fetch, subprocess logic, and trace instrumentation in `account_use_routine`.
- **In Scope**: Quota fetch for the target account using its saved credential file; `resolve_model()` + `resolve_effort()` reuse from `usage.rs`; `run_isolated()` subprocess spawn (always when quota fetch succeeds; subprocess is idempotent — exits immediately if already active); `touch::` (default `1`), `refresh::` (default `1`), `imodel::`, `effort::`, `trace::` (default `0`) parameter registration on `.account.use`; graceful skip when quota fetch fails; OAuth token refresh when locally expired and `refresh::1`; timestamped `account.use` diagnostic lines covering credential read, quota fetch, subprocess scheduling, model/effort resolution, subprocess dispatch, and refresh attempt.
- **Out of Scope**: `.usage` subprocess control (→ 026_subprocess_model_effort.md); credential rotation mechanics (→ 004_account_use.md).

### Design

After `switch_account()` succeeds, `.account.use` fetches quota data for the target account and activates its 5h session window when quota fetch succeeds. The subprocess is idempotent: it exits immediately when the account is already active. Fix(BUG-285): the idle check (`five_hour.resets_at.is_none()`) was removed — `resets_at` is server-side state set by any session on any machine and cannot proxy local subprocess identity.

**Execution sequence (when `touch::1`, the default):**

1. Resolve and validate `name::` — check account exists in credential store.
2. Fetch quota for the target account from `{credential_store}/{name}.credentials.json` — one HTTP call to `/api/oauth/usage`. If fetch fails (network error, expired token), record failure and continue to step 3.
3. If `dry::1`: print `[dry-run] would switch to '{name}'` (no files changed, no subprocess).
4. `switch_account(name)` — atomic credential rotation (credentials, active marker, best-effort `oauthAccount` patch); model snapshot restore from `{name}.json` into `~/.claude/settings.json`.
4b. If quota fetch succeeded (step 2): call `apply_model_override()` which is now bidirectional (Fix BUG-311). When `seven_day_sonnet` is `Some` and remaining < 10%: overwrite `~/.claude/settings.json` model with `"opus"` (Fix BUG-225; alias fix BUG-257; normalization fix BUG-286). When `seven_day_sonnet` is `Some` and remaining >= 10%: overwrite model with `"sonnet"` if currently in any `"opus"` form (Fix BUG-311). When `seven_day_sonnet` is absent (`None`): overwrite model with `"sonnet"` conservatively — absent tier is treated as unknown, not exhausted (Fix BUG-300; conservative sonnet restore extended by Fix BUG-311). When `touch_ctx` is absent (fetch failed — see BUG-226 limitation), this step is skipped and the snapshot model is installed as-is.
5. If quota fetch succeeded (step 2): call `resolve_model(quota, imodel_param)` → `IsolatedModel`; call `resolve_effort(&model, effort_param)` → `Option<&str>`; call `claude_profile_core::account::refresh_account_token()` with the resolved model and effort args (Feature 017 AC-34 / invariant 008) — which internally applies `expiresAt=1` manipulation (Feature 017 AC-32, RT rotation) and live credential sync for the current account (Feature 017 AC-33, race safety), then invokes `run_isolated` with `["--print", "."]`. The subprocess is idempotent — exits immediately if account is already active. After subprocess completes: re-fetch quota via `fetch_oauth_usage` unconditionally using the saved OAuth token (best-effort; failure does not abort the switch). The re-fetch ensures the account's session state is visible to any subsequent command (e.g., `.usage`) that also evaluates quota timers. (Fix(BUG-285): idle check removed; `AlreadyActive` variant removed from `PreSwitchOutcome`. Fix(BUG-288): post-subprocess re-fetch added.)

**When `touch::0`:** Steps 1, 4, 5 only. No quota fetch, no subprocess. Pure credential rotation (pre-Feature-027 behavior).

**When quota fetch fails:** Touch is skipped silently. Credential rotation still completes and exits 0. Not a fatal error — connectivity issues should not prevent account switching.

**Output:** `switched to '{name}'` on stdout regardless of touch outcome. When `trace::1` and `touch::1`, the following timestamped `account.use` diagnostic lines are emitted to stderr in sequence:

```
2026-06-25 · 16:40:04 · account.use  {name}  reading {cred_path}
2026-06-25 · 16:40:04 · account.use  {name}  reading: OK                                              ← omitted on Err; Err → subprocess: skipped (no further lines)
2026-06-25 · 16:40:04 · account.use  {name}  quota fetch: OK                                          ← or Err({msg}); Err → subprocess: skipped + optional expiry check
2026-06-25 · 16:40:04 · account.use  {name}  subprocess: skipped (reason: fetch failed)               ← fetch Err path only; always precedes expiry check line
2026-06-25 · 16:40:04 · account.use  {name}  expiry check: expired(4h 21m ago) → attempting refresh  ← refresh::1 (default) + expired; refresh attempt follows
2026-06-25 · 16:40:04 · account.use  {name}  expiry check: refresh OK — re-probing touch context      ← refresh succeeded; switch continues
                    OR: expiry check: refresh failed → refused                                         ← refresh failed; exits 3
                    OR: expiry check: expired(4h 21m ago) → refused (refresh::0)                      ← refresh::0; exits 3 immediately
                    OR: expiry check: valid (expires in 3h 34m)                                        ← expiresAt in future; switch continues; omitted when expiresAt absent
2026-06-25 · 16:40:04 · account.use  {name}  subprocess: scheduled (idle check removed)              ← fetch OK path only; BUG-285: idle check removed
2026-06-25 · 16:40:04 · account.use  {name}  model override: sonnet→opus (7d(Son) left=5%)           ← fetch OK + 7d(Son) < 10% + snapshot was Sonnet (any form) or full-ID opus; omitted otherwise
2026-06-25 · 16:40:04 · account.use  {name}  model: {model}  effort: {effort}                        ← fetch OK path only
2026-06-25 · 16:40:04 · account.use  {name}  subprocess: spawned                                      ← fetch OK path only
```

When `trace::1` and `touch::0`: no timestamped `account.use` diagnostic lines (no fetch operations performed). When `trace::0` (default): no trace output.

**Model/effort resolution:** Delegates entirely to `resolve_model()` and `resolve_effort()` in `usage.rs`. All semantics from Feature 026 apply unchanged:
- `imodel::auto` (default): `claude-sonnet-5` when `son_idle=true` (`seven_day_sonnet` present and `resets_at=None`); otherwise `claude-haiku-4-5-20251001` — see Feature 026 for full algorithm
- `effort::auto` (default): `low` for any model that supports effort; no flag for `imodel::keep` or `imodel::haiku`

**Layer assignment:** Quota fetch and subprocess call are added to `account_use_routine()` in `commands.rs`. Resolution functions (`resolve_model`, `resolve_effort`) are reused from `usage.rs` with no changes.

**Exit codes:**
- 0: success (switch completed; subprocess spawned if fetch succeeded, skipped if fetch failed)
- 1: usage error (invalid name format or invalid `imodel::`/`effort::` value)
- 2: runtime error (account not found or HOME unset)
- 3: account credentials expired AND refresh failed (or `refresh::0`) — locally-expired `expiresAt` detected when `touch::1` and quota fetch fails; refresh attempted first when `refresh::1` (see AC-17, AC-20)

### Acceptance Criteria

- **AC-01**: `clp .account.use name::alice@home.com` (default `touch::1`) fetches quota for the target account, switches credentials, spawns `run_isolated` when quota fetch succeeds (subprocess is idempotent), and re-fetches quota post-subprocess so the account's session state is reflected immediately (Fix(BUG-285): idle check removed; Fix(BUG-288): post-subprocess re-fetch added).
- **AC-02**: `clp .account.use name::alice@home.com touch::0` performs pure credential rotation with no quota fetch and no subprocess.
- **AC-03**: `clp .account.use` against an already-active account (`resets_at.is_some()`) spawns the subprocess idempotently; subprocess exits immediately; overall command exits 0. Fix(BUG-285): the idle check that previously skipped the subprocess for already-active accounts used server-side `resets_at` as a local subprocess identity oracle (category error).
- **AC-04**: When the quota fetch fails (network error, auth error) AND the target account's token is NOT locally expired (`expiresAt` absent or in the future): touch is skipped silently and the switch still completes; exits 0.
- **AC-05**: `imodel::auto` selects `claude-sonnet-5` when `son_idle=true` (`seven_day_sonnet` field present and `resets_at=None`); otherwise selects `claude-haiku-4-5-20251001`. Delegates to `resolve_model()` — full algorithm documented in Feature 026.
- **AC-06**: `effort::auto` injects `--effort low` for any model that supports effort; no `--effort` flag for `imodel::keep` or `imodel::haiku`.
- **AC-07**: `imodel::bad` exits 1 with stderr naming `auto`, `sonnet`, `opus`, `haiku`, `keep`; `effort::bad` exits 1 with stderr naming `auto`, `low`, `normal`, `high`, `max`.
- **AC-08**: `dry::1` prints `[dry-run] would switch to '{name}'` without modifying credentials or spawning a subprocess.
- **AC-09**: `touch::`, `refresh::`, `imodel::`, `effort::`, and `trace::` appear in `.account.use --help` output with their defaults (`1`, `1`, `auto`, `auto`, `0`).
- **AC-10**: When `trace::1` and `touch::1`: emits `... · account.use  {name}  reading {path}` followed by `reading: OK` on success; if the credential file read fails, emits `reading: Err({msg})` and stops — no further trace lines for this invocation.
- **AC-11**: When `trace::1` and `touch::1` and credential read succeeds: emits `... · account.use  {name}  quota fetch: OK` on success or `quota fetch: Err({msg})` on failure; Err stops model/subprocess trace lines and triggers the expiry check (AC-17) when `expiresAt` is present in the credential file.
- **AC-12**: When `trace::1` and `touch::1` and quota fetch succeeds: emits `... · account.use  {name}  subprocess: scheduled (idle check removed)`. The idle-check trace lines (`idle check: resets_at=...`) no longer exist — Fix(BUG-285) removed the idle check entirely.
- **AC-13**: When `trace::1` and `touch::1` and quota fetch succeeded: emits `... · account.use  {name}  model: {model}  effort: {effort}` before `subprocess: spawned`. Omitted only when fetch failed.
- **AC-14**: When `trace::1` and `touch::1`: emits `... · account.use  {name}  subprocess: spawned` when fetch succeeded, or `subprocess: skipped (reason: fetch failed)` when fetch fails. `subprocess: skipped (reason: already active)` no longer exists (Fix(BUG-285)).
- **AC-15**: When `trace::1` and `touch::0`: no timestamped `account.use` diagnostic lines emitted — no quota fetch operations are performed on the `touch::0` path.
- **AC-16**: `trace::` (Kind::String, default `0`) is registered on `.account.use`; `trace::bad` exits 1 with stderr naming `0`, `1`, `false`, `true`.
- **AC-17**: When `touch::1` (default) and the quota fetch fails (returns Err) AND the target account's token is locally expired (current time > `expiresAt` from `{credential_store}/{name}.credentials.json`) AND `refresh::1` (default): first calls `attempt_expired_token_refresh()`. If refresh succeeds: re-probes touch context with fresh token; continues with the switch (exits 0 on success). If refresh fails: exits 3 with `account credentials expired and refresh failed: {name} (expired {N}h {M}m ago)` on stderr; `switch_account()` is NOT called. When `trace::1`: emits `expired({N}h {M}m ago) → attempting refresh` → then `refresh OK — re-probing touch context` or `refresh failed → refused`. (Fix for BUG-213; extended by BUG-230.)
- **AC-18**: When quota fetch succeeded (step 2), `apply_model_override()` runs bidirectionally before the subprocess step (Fix BUG-311). (a) When `seven_day_sonnet` is `Some` and remaining < 10%: if session model contains `"sonnet"` (both shorthand and full-ID `"claude-sonnet-5"`) or is absent or equals full-ID `"claude-opus-4-8"`, overwrites model with `"opus"` shorthand. (Fix BUG-225; alias fix BUG-257; normalization fix BUG-286.) (b) When `seven_day_sonnet` is `Some` and remaining >= 10%: if session model is any `"opus"` form (shorthand, full-ID, or empty), overwrites model with `"sonnet"` — restores Sonnet when quota has recovered (Fix BUG-311). (c) When `seven_day_sonnet` is absent (`None`): writes `"sonnet"` conservatively if model is any `"opus"` form — absent tier is not exhausted (Fix BUG-300; conservative restore extended by Fix BUG-311). `override_session_model_to_sonnet()` is a no-op when model is already `"sonnet"` shorthand.
- **AC-19**: When `trace::1` and the model override fires (AC-18): emits `... · account.use  {name}  model override: sonnet→opus (7d(Son) left={N}%)` before the `model:` line when upgrading to Opus. Emits `... · account.use  {name}  model override: opus→sonnet (7d(Son) left={N}%)` when restoring to Sonnet (Sonnet tier present and left >= 10%, Fix BUG-311). Omitted when no model change occurs (model already correct, tier absent, or quota fetch failed).
- **AC-20**: When `touch::1` (default) and the quota fetch fails AND the target token is locally expired AND `refresh::0`: exits 3 immediately with `account credentials expired: {name} (expired {N}h {M}m ago)` on stderr; no refresh attempt is made. When `trace::1`: emits `expired({N}h {M}m ago) → refused (refresh::0)`. (Fix for BUG-230.)
- **AC-21**: After the `run_isolated` subprocess completes, `apply_post_switch_touch` re-fetches quota for the switched-to account using its saved OAuth token (`fetch_oauth_usage`). The re-fetch is best-effort (non-aborting) — it fires regardless of whether the subprocess returned new credentials, and silently skips on any failure (unreadable credential file, missing `accessToken`, or HTTP error). On success, the in-memory quota result reflects the post-subprocess state, so any subsequent `.usage touch` call will see the active `resets_at` values and skip redundant subprocess spawning. On failure, the pre-subprocess quota data remains; the switch and subprocess are not rolled back. This mirrors `apply_touch` AC-03 in Feature 024. Fix(BUG-288): absence of this re-fetch caused `.usage touch` to see stale pre-subprocess quota (`resets_at = None`) and spawn a redundant second subprocess for the just-switched account.
- **Limitation (BUG-226)**: When the quota fetch returns 429 (rate-limited) or any other error, quota data is unavailable and the quota-aware model upgrade (AC-18) cannot fire. The snapshot model restored by `switch_account()` is installed as-is — potentially leaving the session on Sonnet even when Sonnet quota is exhausted. No workaround exists at the `.account.use` layer; the user must manually override via `imodel::opus`.

### Bugs

| File | Relationship |
|------|--------------|
| BUG-213 | BUG-213 ✅ Fixed by TSK-216: expiry guard inserted in `account_use_routine()` before `switch_account()`; exits 3 when `now_ms > expiresAt` on the fetch-failed path |
| BUG-238 | BUG-238 ✅ Fixed: `pre_switch_touch_ctx()` refactored to `PreSwitchOutcome` enum; `apply_model_override()` extracted and called for all NeedTouch outcomes. Note: the AlreadyActive variant was subsequently removed by BUG-285. |
| BUG-285 | BUG-285 ✅ Fixed: idle check (`resets_at.is_none()`) removed from `pre_switch_touch_ctx`; `AlreadyActive` variant removed from `PreSwitchOutcome`; subprocess always fires when fetch succeeds |
| BUG-257 | BUG-257 ✅ Fixed (TSK-261): `contains("sonnet")` + write `"opus"` shorthand |
| BUG-286 | BUG-286 🟢 Fixed (TSK-261): `override_session_model_to_opus` gate extended to match `"claude-opus-4-8"` (full-ID opus) in addition to Sonnet forms; normalized to `"opus"` shorthand when 7d(Son) < 10% — see AC-18 |
| BUG-288 | BUG-288 🟢 Fixed (Fix A): `apply_post_switch_touch` now re-fetches quota post-subprocess via `fetch_oauth_usage` and persists via `write_quota_cache(paths.base(), name, h5, d7, sn)` — AC-21 satisfied; subsequent `.usage touch` sees updated `resets_at` and skips the redundant subprocess. Fix B (`touch_idle` read site in `apply_touch`) deferred to follow-on task. |
| BUG-300 | BUG-300 ✅ Fixed (TSK-302): `apply_model_override()` used `map_or(0.0, ...)` on `quota.seven_day_sonnet`; when `None`, returned 0.0 < 20.0 → Opus override fired unconditionally for accounts without a Sonnet tier. Fix: `if let Some(ref sonnet) = quota.seven_day_sonnet` guard at `api.rs:267`; `mre_bug300_model_override_absent_sonnet_no_override` added to `api_tests.rs` |
| BUG-311 | BUG-311 ✅ Fixed (2026-06-23): `apply_model_override()` was one-way — only wrote `"opus"` when exhausted; no code wrote `"sonnet"` back when quota recovered. Settings.json retained stale `"opus"` after switching to an account with sufficient Sonnet quota. Fix: added `override_session_model_to_sonnet()` in `claude_profile_core`; added else-branch (sonnet-sufficient) + tier-absent else to `apply_model_override()` (AC-18, AC-19). |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md#command-5-accountuse) | `.account.use` CLI specification |

### Dependencies

| File | Relationship |
|------|--------------|
| `claude_runner_core` | `run_isolated()`, `IsolatedModel` — subprocess execution (invoked internally by `refresh_account_token`) |
| `claude_profile_core` | `refresh_account_token()` — single token refresh entry point (invariant 008) |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/008_single_token_refresh_entry.md](../invariant/008_single_token_refresh_entry.md) | Invariant 008: all token refresh through `refresh_account_token()` — AC-34 routes `apply_post_switch_touch` through this entry point (TSK-320) |

### Features

| File | Relationship |
|------|--------------|
| [004_account_use.md](004_account_use.md) | Credential rotation mechanics — prerequisite step |
| [024_session_touch.md](024_session_touch.md) | Touch subprocess trigger conditions and idle-session semantics |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Model/effort resolution algorithm (`resolve_model`, `resolve_effort`) |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | Explicit session model override — `set_model::` on `.account.use` bypasses `apply_model_override()` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/023_trace.md](../cli/param/023_trace.md) | `trace::` parameter specification (shared with `.usage`) |
| [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |
| [cli/param/035_imodel.md](../cli/param/035_imodel.md) | `imodel::` parameter specification (shared with `.usage`) |
| [cli/param/036_effort.md](../cli/param/036_effort.md) | `effort::` parameter specification (shared with `.usage`) |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.use`](../cli/command/001_account.md#command-5-accountuse) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/account_ops.rs` | `account_use_routine()` — adds quota fetch + subprocess call after credential rotation |
| `src/lib.rs` | `touch::`, `imodel::`, `effort::`, `trace::` parameter registration on `.account.use` |
| `src/usage/subprocess.rs`, `src/usage/api.rs` | `resolve_model()`, `resolve_effort()` reused from Feature 026; new: `TouchCtx`, `validate_imodel_str()`, `validate_effort_str()`, `pre_switch_touch_ctx()`, `apply_post_switch_touch()`, `apply_model_override()` |
| `claude_profile_core/src/account.rs` | `refresh_account_token()` — invoked by `apply_post_switch_touch()` per Feature 017 AC-34 / invariant 008 |

### Algorithm Docs

| File | Relationship |
|------|-------------|
| [algorithm/001_touch_model_selection.md](../algorithm/001_touch_model_selection.md) | `resolve_model()` decision table — selects Sonnet or Haiku for post-switch subprocess |
| [algorithm/002_session_model_override.md](../algorithm/002_session_model_override.md) | `apply_model_override()` bidirectional logic — Sonnet/Opus written to `settings.json` based on quota |

### Subprocess Docs

| File | Relationship |
|------|-------------|
| [subprocess/004_session_touch_invocation.md](../subprocess/004_session_touch_invocation.md) | `apply_touch()` trigger conditions and skip-reason trace codes — invoked via `refresh_account_token()` in step 5 |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` and `effortLevel` fields written by `apply_model_override()` in step 4b |
