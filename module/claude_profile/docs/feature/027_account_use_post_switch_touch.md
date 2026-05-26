# Feature: `.account.use` Post-Switch Touch

### Scope

- **Purpose**: Activate the switched-to account's idle 5h session window immediately after credential rotation, using the same model/effort resolution as `.usage`.
- **Responsibility**: Documents the `touch::`, `imodel::`, and `effort::` parameters on `.account.use` and the post-switch quota fetch and subprocess logic in `account_use_routine`.
- **In Scope**: Quota fetch for the target account using its saved credential file; idle-session check (`five_hour.resets_at.is_none()`); `resolve_model()` + `resolve_effort()` reuse from `usage.rs`; `run_isolated()` subprocess spawn; `touch::` (default `1`), `imodel::`, `effort::` parameter registration on `.account.use`; graceful skip when quota fetch fails.
- **Out of Scope**: Refreshing expired tokens on auth error (→ 017_token_refresh.md); `.usage` subprocess control (→ 026_subprocess_model_effort.md); credential rotation mechanics (→ 004_account_use.md); `trace::` diagnostic output for touch steps (not added in this feature).

### Design

After `switch_account()` succeeds, `.account.use` fetches quota data for the target account and activates its 5h session window if idle.

**Execution sequence (when `touch::1`, the default):**

1. Resolve and validate `name::` — check account exists in credential store.
2. Fetch quota for the target account from `{credential_store}/{name}.credentials.json` — one HTTP call to `/api/oauth/usage`. If fetch fails (network error, expired token), record failure and continue to step 4.
3. Determine idle status from fetched data: `five_hour.resets_at.is_none()` → idle; `resets_at.is_some()` → already active.
4. If `dry::1`: print `[dry-run] would switch to '{name}'` (no files changed, no subprocess).
5. `switch_account(name)` — atomic credential rotation (credentials, active marker, best-effort `oauthAccount` patch).
6. If quota fetch succeeded (step 2) AND account was idle (step 3): call `resolve_model(quota, imodel_param)` → `IsolatedModel`; call `resolve_effort(&model, effort_param)` → `Option<&str>`; spawn `run_isolated()` with `["--print", "."]` plus optional `--model` and `--effort` flags.

**When `touch::0`:** Steps 1, 4, 5 only. No quota fetch, no subprocess. Pure credential rotation (pre-Feature-027 behavior).

**When quota fetch fails:** Touch is skipped silently. Credential rotation still completes and exits 0. Not a fatal error — connectivity issues should not prevent account switching.

**Output:** `switched to '{name}'` regardless of touch outcome. Touch activity is not surfaced in normal output (no `trace::` in this feature scope).

**Model/effort resolution:** Delegates entirely to `resolve_model()` and `resolve_effort()` in `usage.rs`. All semantics from Feature 026 apply unchanged:
- `imodel::auto` (default): `claude-sonnet-4-6` if `7d(Son) ≥ 30%`, else `claude-opus-4-6`
- `effort::auto` (default): `high` for Sonnet, `max` for Opus, no flag for `imodel::keep`

**Layer assignment:** Quota fetch and subprocess call are added to `account_use_routine()` in `commands.rs`. Resolution functions (`resolve_model`, `resolve_effort`) are reused from `usage.rs` with no changes.

**Exit codes:**
- 0: success (switch completed; touch fired if idle, skipped if active or fetch failed)
- 1: usage error (invalid name format or invalid `imodel::`/`effort::` value)
- 2: runtime error (account not found or HOME unset)

### Acceptance Criteria

- **AC-01**: `clp .account.use name::alice@home.com` (default `touch::1`) fetches quota for the target account, switches credentials, and spawns `run_isolated` if the account's `five_hour.resets_at` is absent (idle).
- **AC-02**: `clp .account.use name::alice@home.com touch::0` performs pure credential rotation with no quota fetch and no subprocess.
- **AC-03**: `clp .account.use` against an already-active account (`resets_at.is_some()`) completes without spawning a subprocess; exits 0.
- **AC-04**: When the quota fetch fails (network error, auth error), touch is skipped silently and the switch still completes; exits 0.
- **AC-05**: `imodel::auto` selects `claude-sonnet-4-6` when `7d(Son) ≥ 30%` and `claude-opus-4-6` when `7d(Son) < 30%` or unavailable; delegates to `resolve_model()`.
- **AC-06**: `effort::auto` injects `--effort high` for Sonnet and `--effort max` for Opus; no `--effort` flag for `imodel::keep`.
- **AC-07**: `imodel::bad` exits 1 with stderr naming `auto`, `sonnet`, `opus`, `keep`; `effort::bad` exits 1 with stderr naming `auto`, `high`, `max`.
- **AC-08**: `dry::1` prints `[dry-run] would switch to '{name}'` without modifying credentials or spawning a subprocess.
- **AC-09**: `touch::`, `imodel::`, and `effort::` appear in `.account.use --help` output with their defaults (`1`, `auto`, `auto`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `account_use_routine()` — adds quota fetch + subprocess call after credential rotation |
| source | `src/lib.rs` | `touch::`, `imodel::`, `effort::` parameter registration on `.account.use` |
| source | `src/usage.rs` | `resolve_model()`, `resolve_effort()` reused from Feature 026; new: `TouchCtx`, `validate_imodel_str()`, `validate_effort_str()`, `pre_switch_touch_ctx()`, `apply_post_switch_touch()` |
| dep | `claude_runner_core` | `run_isolated()`, `IsolatedModel` — subprocess execution |
| doc | [004_account_use.md](004_account_use.md) | Credential rotation mechanics — prerequisite step |
| doc | [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Model/effort resolution algorithm (`resolve_model`, `resolve_effort`) |
| doc | [024_session_touch.md](024_session_touch.md) | Touch subprocess trigger conditions and idle-session semantics |
| param | [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |
| param | [cli/param/035_imodel.md](../cli/param/035_imodel.md) | `imodel::` parameter specification (shared with `.usage`) |
| param | [cli/param/036_effort.md](../cli/param/036_effort.md) | `effort::` parameter specification (shared with `.usage`) |
| command | [cli/command/001_account.md](../cli/command/001_account.md#command--5-accountuse) | `.account.use` CLI specification |
