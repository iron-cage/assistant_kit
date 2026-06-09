# Feature: `.account.use` Post-Switch Touch

### Scope

- **Purpose**: Activate the switched-to account's idle 5h session window immediately after credential rotation, using the same model/effort resolution as `.usage`; expose the post-switch touch lifecycle via `trace::` for diagnostics.
- **Responsibility**: Documents the `touch::`, `imodel::`, `effort::`, and `trace::` parameters on `.account.use` and the post-switch quota fetch, subprocess logic, and trace instrumentation in `account_use_routine`.
- **In Scope**: Quota fetch for the target account using its saved credential file; idle-session check (`five_hour.resets_at.is_none()`); `resolve_model()` + `resolve_effort()` reuse from `usage.rs`; `run_isolated()` subprocess spawn; `touch::` (default `1`), `refresh::` (default `1`), `imodel::`, `effort::`, `trace::` (default `0`) parameter registration on `.account.use`; graceful skip when quota fetch fails; OAuth token refresh when locally expired and `refresh::1`; `[trace] account.use` lines covering credential read, quota fetch, idle check, model/effort resolution, subprocess dispatch, and refresh attempt.
- **Out of Scope**: `.usage` subprocess control (→ 026_subprocess_model_effort.md); credential rotation mechanics (→ 004_account_use.md).

### Design

After `switch_account()` succeeds, `.account.use` fetches quota data for the target account and activates its 5h session window if idle.

**Execution sequence (when `touch::1`, the default):**

1. Resolve and validate `name::` — check account exists in credential store.
2. Fetch quota for the target account from `{credential_store}/{name}.credentials.json` — one HTTP call to `/api/oauth/usage`. If fetch fails (network error, expired token), record failure and continue to step 4.
3. Determine idle status from fetched data: `five_hour.resets_at.is_none()` → idle; `resets_at.is_some()` → already active.
4. If `dry::1`: print `[dry-run] would switch to '{name}'` (no files changed, no subprocess).
5. `switch_account(name)` — atomic credential rotation (credentials, active marker, best-effort `oauthAccount` patch); model snapshot restore from `{name}.json` into `~/.claude/settings.json`.
5b. If quota fetch succeeded (step 2): check `seven_day_sonnet` utilization; if remaining < 20% and restored model is Sonnet (any form: `"sonnet"`, `"claude-sonnet-4-6"`, or absent), overwrite `~/.claude/settings.json` model with `"opus"` (BUG-225 fix; BUG-257 alias fix). When `touch_ctx` is absent (fetch failed — see BUG-226 limitation), this step is skipped and the snapshot model is installed as-is.
6. If quota fetch succeeded (step 2) AND account was idle (step 3): call `resolve_model(quota, imodel_param)` → `IsolatedModel`; call `resolve_effort(&model, effort_param)` → `Option<&str>`; spawn `run_isolated()` with `["--print", "."]` plus optional `--model` and `--effort` flags.

**When `touch::0`:** Steps 1, 4, 5 only. No quota fetch, no subprocess. Pure credential rotation (pre-Feature-027 behavior).

**When quota fetch fails:** Touch is skipped silently. Credential rotation still completes and exits 0. Not a fatal error — connectivity issues should not prevent account switching.

**Output:** `switched to '{name}'` on stdout regardless of touch outcome. When `trace::1` and `touch::1`, the following `[trace] account.use` lines are emitted to stderr in sequence:

```
[trace] account.use  {name}  reading {cred_path}
[trace] account.use  {name}  reading: OK                                              ← omitted on Err; Err → subprocess: skipped (no further lines)
[trace] account.use  {name}  quota fetch: OK                                          ← or Err({msg}); Err → subprocess: skipped + optional expiry check
[trace] account.use  {name}  subprocess: skipped (reason: fetch failed)               ← fetch Err path only; always precedes expiry check line
[trace] account.use  {name}  expiry check: expired(4h 21m ago) → attempting refresh  ← refresh::1 (default) + expired; refresh attempt follows
[trace] account.use  {name}  expiry check: refresh OK — re-probing touch context      ← refresh succeeded; switch continues
           OR: expiry check: refresh failed → refused                                 ← refresh failed; exits 3
           OR: expiry check: expired(4h 21m ago) → refused (refresh::0)              ← refresh::0; exits 3 immediately
           OR: expiry check: valid (expires in 3h 34m)                               ← expiresAt in future; switch continues; omitted when expiresAt absent
[trace] account.use  {name}  idle check: resets_at=absent → idle                     ← fetch OK path only; or resets_at=present → already active
[trace] account.use  {name}  model override: sonnet→opus (7d(Son) left=5%)           ← fetch OK + 7d(Son) < 20% + snapshot was Sonnet (any form); omitted otherwise
[trace] account.use  {name}  model: {model}  effort: {effort}                        ← fetch OK path only
[trace] account.use  {name}  subprocess: spawned                                      ← or skipped (reason: already active); fetch OK path only
```

When `trace::1` and `touch::0`: no `[trace] account.use` lines (no fetch operations performed). When `trace::0` (default): no trace output.

**Model/effort resolution:** Delegates entirely to `resolve_model()` and `resolve_effort()` in `usage.rs`. All semantics from Feature 026 apply unchanged:
- `imodel::auto` (default): `claude-sonnet-4-6` if `7d(Son) ≥ 20%`, else `claude-opus-4-6`
- `effort::auto` (default): `low` for any model that supports effort; no flag for `imodel::keep` or `imodel::haiku`

**Layer assignment:** Quota fetch and subprocess call are added to `account_use_routine()` in `commands.rs`. Resolution functions (`resolve_model`, `resolve_effort`) are reused from `usage.rs` with no changes.

**Exit codes:**
- 0: success (switch completed; touch fired if idle, skipped if active or fetch failed)
- 1: usage error (invalid name format or invalid `imodel::`/`effort::` value)
- 2: runtime error (account not found or HOME unset)
- 3: account credentials expired AND refresh failed (or `refresh::0`) — locally-expired `expiresAt` detected when `touch::1` and quota fetch fails; refresh attempted first when `refresh::1` (see AC-17, AC-20)

### Acceptance Criteria

- **AC-01**: `clp .account.use name::alice@home.com` (default `touch::1`) fetches quota for the target account, switches credentials, and spawns `run_isolated` if the account's `five_hour.resets_at` is absent (idle).
- **AC-02**: `clp .account.use name::alice@home.com touch::0` performs pure credential rotation with no quota fetch and no subprocess.
- **AC-03**: `clp .account.use` against an already-active account (`resets_at.is_some()`) completes without spawning a subprocess; exits 0.
- **AC-04**: When the quota fetch fails (network error, auth error) AND the target account's token is NOT locally expired (`expiresAt` absent or in the future): touch is skipped silently and the switch still completes; exits 0.
- **AC-05**: `imodel::auto` selects `claude-sonnet-4-6` when `7d(Son) ≥ 20%` and `claude-opus-4-6` when `7d(Son) < 20%` or unavailable; delegates to `resolve_model()`.
- **AC-06**: `effort::auto` injects `--effort low` for any model that supports effort; no `--effort` flag for `imodel::keep` or `imodel::haiku`.
- **AC-07**: `imodel::bad` exits 1 with stderr naming `auto`, `sonnet`, `opus`, `haiku`, `keep`; `effort::bad` exits 1 with stderr naming `auto`, `low`, `normal`, `high`, `max`.
- **AC-08**: `dry::1` prints `[dry-run] would switch to '{name}'` without modifying credentials or spawning a subprocess.
- **AC-09**: `touch::`, `refresh::`, `imodel::`, `effort::`, and `trace::` appear in `.account.use --help` output with their defaults (`1`, `1`, `auto`, `auto`, `0`).
- **AC-10**: When `trace::1` and `touch::1`: emits `[trace] account.use  {name}  reading {path}` followed by `reading: OK` on success; if the credential file read fails, emits `reading: Err({msg})` and stops — no further trace lines for this invocation.
- **AC-11**: When `trace::1` and `touch::1` and credential read succeeds: emits `[trace] account.use  {name}  quota fetch: OK` on success or `quota fetch: Err({msg})` on failure; Err stops idle-check and model/subprocess trace lines and triggers the expiry check (AC-17) when `expiresAt` is present in the credential file.
- **AC-12**: When `trace::1` and `touch::1` and quota fetch succeeds: emits `[trace] account.use  {name}  idle check: resets_at=present → already active` (active path) or `idle check: resets_at=absent → idle` (idle path).
- **AC-13**: When `trace::1` and `touch::1` and quota fetch succeeded: emits `[trace] account.use  {name}  model: {model}  effort: {effort}` (using the resolved model and effort strings) regardless of idle state — appears before `subprocess: spawned` (idle) and before `subprocess: skipped (reason: already active)` (active). Omitted only when fetch failed.
- **AC-14**: When `trace::1` and `touch::1`: emits `[trace] account.use  {name}  subprocess: spawned` when the subprocess is dispatched (idle + fetch OK), or `subprocess: skipped (reason: already active)` / `subprocess: skipped (reason: fetch failed)` otherwise.
- **AC-15**: When `trace::1` and `touch::0`: no `[trace] account.use` lines emitted — no quota fetch operations are performed on the `touch::0` path.
- **AC-16**: `trace::` (Kind::String, default `0`) is registered on `.account.use`; `trace::bad` exits 1 with stderr naming `0`, `1`, `false`, `true`.
- **AC-17**: When `touch::1` (default) and the quota fetch fails (returns Err) AND the target account's token is locally expired (current time > `expiresAt` from `{credential_store}/{name}.credentials.json`) AND `refresh::1` (default): first calls `attempt_expired_token_refresh()`. If refresh succeeds: re-probes touch context with fresh token; continues with the switch (exits 0 on success). If refresh fails: exits 3 with `account credentials expired and refresh failed: {name} (expired {N}h {M}m ago)` on stderr; `switch_account()` is NOT called. When `trace::1`: emits `expired({N}h {M}m ago) → attempting refresh` → then `refresh OK — re-probing touch context` or `refresh failed → refused`. (Fix for BUG-213; extended by BUG-230.)
- **AC-18**: When quota fetch succeeded (step 2) and `seven_day_sonnet` remaining < 20% and the just-restored session model contains `"sonnet"` (matches both shorthand `"sonnet"` and full ID `"claude-sonnet-4-6"`) or is absent: overwrites `~/.claude/settings.json` model with `"opus"` before the subprocess step. (Fix for BUG-225; alias fix for BUG-257.)
- **AC-19**: When `trace::1` and the model override fires (AC-18): emits `[trace] account.use  {name}  model override: sonnet→opus (7d(Son) left={N}%)` before the `model:` line. Omitted when override does not fire (model was already Opus) or when quota fetch failed.
- **AC-20**: When `touch::1` (default) and the quota fetch fails AND the target token is locally expired AND `refresh::0`: exits 3 immediately with `account credentials expired: {name} (expired {N}h {M}m ago)` on stderr; no refresh attempt is made. When `trace::1`: emits `expired({N}h {M}m ago) → refused (refresh::0)`. (Fix for BUG-230.)
- **Limitation (BUG-226)**: When the quota fetch returns 429 (rate-limited) or any other error, quota data is unavailable and the quota-aware model upgrade (AC-18) cannot fire. The snapshot model restored by `switch_account()` is installed as-is — potentially leaving the session on Sonnet even when Sonnet quota is exhausted. No workaround exists at the `.account.use` layer; the user must manually override via `imodel::opus`.

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/213_account_use_switches_to_expired_token_silently.md` | BUG-213 ✅ Fixed by TSK-216: expiry guard inserted in `account_use_routine()` before `switch_account()`; exits 3 when `now_ms > expiresAt` on the fetch-failed path |
| `task/claude_profile/bug/238_model_override_skipped_when_already_active.md` | BUG-238 ✅ Fixed: `pre_switch_touch_ctx()` refactored to `PreSwitchOutcome` enum; `apply_model_override()` extracted and called for both idle and already-active paths |
| `task/claude_profile/bug/257_override_session_model_exact_match_misses_shorthand_alias.md` | BUG-257 ✅ Fixed (TSK-261): `contains("sonnet")` + write `"opus"` shorthand |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md#command--5-accountuse) | `.account.use` CLI specification |

### Dependencies

| File | Relationship |
|------|--------------|
| `claude_runner_core` | `run_isolated()`, `IsolatedModel` — subprocess execution |

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

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/account_ops.rs` | `account_use_routine()` — adds quota fetch + subprocess call after credential rotation |
| `src/lib.rs` | `touch::`, `imodel::`, `effort::`, `trace::` parameter registration on `.account.use` |
| `src/usage/subprocess.rs`, `src/usage/api.rs` | `resolve_model()`, `resolve_effort()` reused from Feature 026; new: `TouchCtx`, `validate_imodel_str()`, `validate_effort_str()`, `pre_switch_touch_ctx()`, `apply_post_switch_touch()`, `apply_model_override()` |
