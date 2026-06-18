# Test: Feature 027 — `.account.use` Post-Switch Touch

Feature behavioral requirement test cases for `docs/feature/027_account_use_post_switch_touch.md`. Each FT case maps to one acceptance criterion. Command-level integration tests (IT-N) are in [cli/command/005_account_use.md](../cli/command/05_account_use.md) (IT-17 through IT-23). Model/effort resolution unit tests are in [feature/026_subprocess_model_effort.md](026_subprocess_model_effort.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::1` idle account → subprocess dispatched after switch | AC-01 | Live |
| FT-02 | `touch::0` idle account → pure rotation, no subprocess | AC-02 | Integration |
| FT-03 | Active account (`resets_at` present) → subprocess spawned idempotently (Fix BUG-285) | AC-03 | Live |
| FT-04 | Quota fetch failure + token NOT expired → touch skipped, switch completes, exits 0 | AC-04 | Integration |
| FT-05 | `imodel::auto` model selection delegates to `resolve_model()` | AC-05 | Structural (→ Feature 026) |
| FT-06 | `effort::auto` effort injection delegates to `resolve_effort()` | AC-06 | Structural (→ Feature 026) |
| FT-07 | `imodel::bad` exits 1 naming all five valid values | AC-07 | Integration |
| FT-08 | `effort::bad` exits 1 naming all five valid values | AC-07 | Integration |
| FT-09 | `dry::1` — no credentials modified, no subprocess spawned | AC-08 | Integration |
| FT-10 | `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` with defaults | AC-09, AC-16 | Integration |
| FT-11 | `trace::1 touch::1` idle account — all 6 trace lines emitted in order | AC-10, AC-11, AC-12, AC-13, AC-14 | Integration |
| FT-12 | `trace::1 touch::1` active account — read+fetch+scheduled+model+spawned lines (no idle-check, BUG-285) | AC-10, AC-11, AC-12, AC-13, AC-14 | Integration |
| FT-13 | `trace::1 touch::1` fetch failure + `expiresAt` future — fetch-err + expiry-valid emitted; idle/model omitted | AC-10, AC-11, AC-14 | Integration |
| FT-14 | `trace::1 touch::0` — no `[trace] account.use` lines emitted | AC-15 | Integration |
| FT-15 | `trace::0` (default) — no `[trace] account.use` lines emitted | AC-15 | Integration |
| FT-16 | `trace::` with bad value exits 1 | AC-16 | Integration |
| FT-17 | `touch::1` + fetch Err + expired `expiresAt` + `refresh::1` → refresh fails → exits 3; switch NOT executed | AC-17 | Integration (BUG-213 + BUG-230 MRE) |
| FT-18 | `touch::1` + fetch Err + expired `expiresAt` + `refresh::0` → exits 3 immediately; no refresh attempt | AC-20 | Integration (BUG-230) |
| FT-19 | Active account + 7d(Son) < 20% → model override fires after switch | AC-18 | Integration (BUG-238 MRE) |
| FT-20 | `override_session_model_to_opus()` fires for shorthand `"sonnet"` input, writes `"opus"` | AC-18 | Unit (BUG-257 MRE) |
| FT-21 | Post-subprocess re-fetch updates in-memory quota; failure preserves pre-subprocess data | AC-21 | Unit (BUG-288 MRE) |
| FT-22 | `seven_day_sonnet = None` → model override does not fire; absent tier treated as unknown, not exhausted | AC-18 | Unit (BUG-300 MRE) |
| — | `trace::1` + model override fires → `model override: sonnet→opus` trace line emitted | AC-19 | Live-only (requires `trace::1` + `7d(Son) < 20%` + Sonnet model in snapshot) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-22 | seven_day_sonnet = None → model override does not fire; absent tier treated as unknown | AC-18 | BUG-300 MRE |
| FT-20 | override_session_model_to_opus fires for shorthand "sonnet" input, writes "opus" | AC-18 | BUG-257 MRE |
| FT-21 | post-subprocess re-fetch updates in-memory quota; failure preserves pre-subprocess data | AC-21 | BUG-288 MRE |
| FT-01 | touch::1 idle account dispatches subprocess | AC-01 | Touch Dispatch |
| FT-02 | touch::0 suppresses subprocess and quota fetch | AC-02 | Touch Suppression |
| FT-03 | active account — subprocess spawned idempotently (BUG-285 fix) | AC-03 | Idle Guard |
| FT-04 | fetch failure + not expired — skip silently, exits 0 | AC-04 | Graceful Degradation |
| FT-05 | imodel::auto delegates to resolve_model() | AC-05 | Model Delegation |
| FT-06 | effort::auto delegates to resolve_effort() | AC-06 | Effort Delegation |
| FT-07 | imodel::bad exits 1 with valid values | AC-07 | Rejection |
| FT-08 | effort::bad exits 1 with valid values | AC-07 | Rejection |
| FT-09 | dry::1 performs no modification | AC-08 | Dry Run |
| FT-10 | touch:: refresh:: imodel:: effort:: trace:: in help with defaults | AC-09, AC-16 | Help Output |
| FT-11 | trace::1 touch::1 account — subprocess always dispatched when fetch OK, 6 trace lines emitted | AC-10, AC-11, AC-12, AC-13, AC-14 | Trace Output |
| FT-12 | trace::1 touch::1 active account — read+fetch+scheduled+model+spawned lines (no idle-check, BUG-285) | AC-10, AC-11, AC-12, AC-13, AC-14 | Trace Output |
| FT-13 | trace::1 touch::1 fetch failure + expiresAt future — fetch-err + expiry-valid lines; idle/model omitted | AC-10, AC-11, AC-14 | Trace Output |
| FT-14 | trace::1 touch::0 — no trace lines emitted | AC-15 | Trace Suppression |
| FT-15 | trace::0 (default) — no trace lines emitted | AC-15 | Trace Default |
| FT-16 | trace:: in .account.use --help with default 0 | AC-16 | Help Output |
| FT-17 | touch::1 + fetch Err + expired expiresAt + refresh::1 (default) → refresh fails → exits 3 | AC-17 | BUG-213 + BUG-230 MRE |
| FT-18 | touch::1 + fetch Err + expired expiresAt + refresh::0 → exits 3 immediately, no refresh attempt | AC-20 | BUG-230 |
| FT-19 | active account + 7d(Son) < 20% → model override sonnet→opus after switch | AC-18 | BUG-238 MRE |
| FT-21 | post-subprocess re-fetch updates in-memory quota; failure preserves pre-subprocess data | AC-21 | BUG-288 MRE |

**Total:** 22 FT cases

---

### FT-01: `touch::1` idle account dispatches subprocess after switch

- **Given:** Account `alice@home.com` saved with valid OAuth token and idle 5h window (`five_hour.resets_at` is absent). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; an isolated subprocess (`run_isolated`) is dispatched to start the idle 5h session window.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and idle `five_hour.resets_at = None` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-01](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-02: `touch::0` suppresses quota fetch and subprocess

- **Given:** Account `alice@home.com` saved with idle 5h window.
- **When:** `clp .account.use name::alice@home.com touch::0`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; no quota fetch performed; no subprocess dispatched. Behavior is identical to pre-Feature-027 `.account.use`.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-02](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-03: Active account (`resets_at` present) — subprocess spawned idempotently (Fix BUG-285)

- **Given:** Account `alice@home.com` saved with valid OAuth token and an active 5h window (`five_hour.resets_at` is set).
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; subprocess (`run_isolated`) is spawned — it exits immediately because the account is already active. Fix(BUG-285): the idle check using `resets_at` as a subprocess identity oracle was removed; subprocess always fires when quota fetch succeeds.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and active `five_hour.resets_at` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-03](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-04: Quota fetch failure + `expiresAt` future — touch skipped silently, switch completes

- **Given:** Account `alice@home.com` saved with a credential file that has no `accessToken` field and `expiresAt = FAR_FUTURE_MS` (far-future timestamp, not locally expired). Quota fetch against the saved credential file fails immediately (no `accessToken` → auth error). Because `expiresAt` is in the future, the expiry check passes — this is the non-expired path per AC-04. (See FT-17 for the expired-`expiresAt` path that exits 3.)
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; touch skipped silently. No error output. Fetch failure with a non-expired `expiresAt` is non-fatal.
- **Exit:** 0
- **Source fn:** `aw23_touch_skipped_no_access_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-04](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-05: `imodel::auto` delegates model selection to `resolve_model()`

- **Given:** Feature 026 unit tests cover `resolve_model()` exhaustively (FT-01 through FT-07 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — it calls `resolve_model(&quota, imodel_param)` with the quota fetched for the target account.
- **Then:** Model selection behavior is identical to `.usage` touch path — `imodel::auto` picks Sonnet when `7d(Son) ≥ 20%`, Opus otherwise. All resolution semantics are governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-01..FT-07)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_model` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-05](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-06: `effort::auto` delegates effort injection to `resolve_effort()`

- **Given:** Feature 026 unit tests cover `resolve_effort()` exhaustively (FT-08 through FT-12 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — it calls `resolve_effort(&model, effort_param)` with the resolved model.
- **Then:** Effort injection behavior is identical to `.usage` touch path — `effort::auto` injects `--effort low` for any model, nothing for `imodel::keep` or `imodel::haiku`. All injection semantics governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-08..FT-12)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_effort` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-06](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-07: `imodel::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `aw24_imodel_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-08: `effort::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `aw25_effort_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-09: `dry::1` — no credentials modified, no subprocess spawned

- **Given:** Account `alice@home.com` saved with idle 5h window. Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com dry::1`
- **Then:** Exits 0; stdout contains `[dry-run] would switch to 'alice@home.com'`; credentials file unchanged; active marker unchanged; no subprocess dispatched. The dry-run short-circuit fires before both credential rotation and touch subprocess.
- **Exit:** 0
- **Source fn:** `aw02_switch_dry_run` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-08](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-10: `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` with correct defaults

- **Given:** Standard environment.
- **When:** `clp .account.use --help` (or `.account.use help::1`)
- **Then:** Exits 0. Help output contains `touch::` with default `1`, `refresh::` with default `1`, `imodel::` with default `auto`, `effort::` with default `auto`, and `trace::` with default `0`.
- **Exit:** 0
- **Source fn:** `aw26_help_shows_touch_imodel_effort` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09, AC-16](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-11: `trace::1 touch::1` account — subprocess always dispatched when quota fetch OK

- **Given:** Account `alice@home.com` saved with valid OAuth token. Credential store has a valid `alice@home.com.credentials.json`.
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr (in order): `[trace] account.use  alice@home.com  reading {path}`, `reading: OK`, `quota fetch: OK`, `subprocess: scheduled (idle check removed)`, `model: {model}  effort: {effort}`, `subprocess: spawned`. Fix(BUG-285): `idle check:` trace line removed; subprocess always fires when fetch succeeds.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token)
- **Source fn:** `aw28_lim_it_trace_idle_account_all_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-12: `trace::1 touch::1` account with active 5h window — subprocess spawned (idempotent)

- **Given:** Account `alice@home.com` saved with valid OAuth token and an active 5h window (`five_hour.resets_at` present).
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr contains: `quota fetch: OK`, `subprocess: scheduled (idle check removed)`, `model: {model}  effort: {effort}`, `subprocess: spawned`. Fix(BUG-285): `subprocess: skipped (reason: already active)` no longer emitted; subprocess is always dispatched and exits immediately when already active.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token with `five_hour.resets_at` present in live quota response)
- **Source fn:** `aw29_lim_it_trace_active_account_subprocess_skipped` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-13: `trace::1 touch::1` quota fetch failure + `expiresAt` in future — fetch-err + expiry-valid trace lines

- **Given:** Account `alice@home.com` saved with `accessToken = "invalid-token"` (invalid, causes HTTP auth error) and `expiresAt = FAR_FUTURE_MS` (not expired). Quota fetch fails with an auth error. Because `expiresAt` is in the future, the expiry check passes and emits a `valid` trace line — the switch completes. (See FT-17 for the expired-`expiresAt` path that exits 3.)
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr contains (in order): `reading {path}`, `reading: OK`, `quota fetch: Err({msg})`, `subprocess: skipped (reason: fetch failed)`, `expiry check: valid (expires in`. No `idle check:` or `model:` lines emitted.
- **Exit:** 0
- **Source fn:** `aw30_trace_fetch_failure_skips_idle_model_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-14: `trace::1 touch::0` — no `[trace] account.use` lines emitted

- **Given:** Any account store state.
- **When:** `clp .account.use name::alice@home.com touch::0 trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr: no `[trace] account.use` lines (no quota fetch operations performed).
- **Exit:** 0
- **Source fn:** `aw31_trace_touch_disabled_no_trace_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-15: `trace::0` (default) — no `[trace] account.use` lines emitted

- **Given:** Account `alice@home.com` saved with valid credentials and idle 5h window.
- **When:** `clp .account.use name::alice@home.com` (default `trace::0`, default `touch::1`)
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr: no `[trace] account.use` lines. This is the standard non-diagnostic output path.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` — already covers absence of trace output for non-trace invocations
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-16: `trace::` with bad value exits 1

- **Given:** Any account store state.
- **When:** `clp .account.use name::alice@home.com trace::bad`
- **Then:** Exits 1. Stderr names the four valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `aw32_trace_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-16](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-17: `touch::1` + fetch Err + expired `expiresAt` + `refresh::1` — refresh fails → exits 3 (BUG-213 + BUG-230 MRE)

- **Given:** Account `alice@home.com` saved with a credential file where `expiresAt` is set to a timestamp in the past (locally expired token) and no `accessToken` (so the refresh subprocess immediately fails). Default `refresh::1` applies.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1 refresh::1`)
- **Then:** Exits 3. Stderr contains `account credentials expired and refresh failed: alice@home.com (expired ...ago)`. `~/.claude/.credentials.json` is NOT overwritten. The active marker is NOT updated.
- **Exit:** 3
- **Source fn:** `mre_bug213_account_use_refuses_expired_token_on_fetch_error` + `mre_bug230_account_use_refresh_fails_exits_3_with_updated_message` (in `tests/cli/account_mutations_test.rs`)
- **Note:** BUG-213 MRE still passes — `err.contains("account credentials expired")` holds because the new message `"account credentials expired and refresh failed"` is a superset. The BUG-230 MRE additionally asserts `err.contains("and refresh failed")`. For `refresh::0` (immediate exit), see FT-18. The discriminant between FT-04 and FT-17 is the `expires_at_ms` argument to `write_account()`: `FAR_FUTURE_MS` (future, not expired) vs `1000` (past, expired).
- **Source:** [feature/027_account_use_post_switch_touch.md AC-17](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-18: `touch::1` + fetch Err + expired `expiresAt` + `refresh::0` — exits 3 immediately (BUG-230)

- **Given:** Account `alice@home.com` saved with a credential file where `expiresAt` is in the past and no `accessToken`. `refresh::0` explicitly disables the refresh attempt.
- **When:** `clp .account.use name::alice@home.com refresh::0 trace::1`
- **Then:** Exits 3. Stderr contains `account credentials expired: alice@home.com (expired ...ago)`. Does NOT contain `"and refresh failed"` (no refresh attempted). Trace contains `refused (refresh::0)`. `~/.claude/.credentials.json` is NOT overwritten.
- **Exit:** 3
- **Source fn:** `aw33_refresh_disabled_exits_3_immediately` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-20](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-19: Active account + 7d(Son) < 20% — model override sonnet→opus fires after switch (BUG-238 MRE)

- **Given:** Account `alice@home.com` saved with valid OAuth token and an ACTIVE 5h window (`five_hour.resets_at` is set). `seven_day_sonnet.utilization > 80%` (remaining < 20%). The account's `{name}.json` contains `{"model": "sonnet"}` (shorthand — the production convention Claude Code uses in `settings.json`; see BUG-257).
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0. `switched to 'alice@home.com'` on stdout. After the switch, `~/.claude/settings.json` contains `"model": "opus"` (shorthand — BUG-257 write-side fix; `override_session_model_to_opus()` now writes the shorthand convention). The BUG-225 Sonnet→Opus override fires even though the account is already active (no subprocess spawned, but model override still applied). Before the BUG-238 fix: model stayed at `"sonnet"` because `pre_switch_touch_ctx()` returned `None` for active accounts, skipping the override.
- **Exit:** 0
- **Source fn:** `mre_bug238_model_override_fires_for_active_account` (in `src/usage/api_tests.rs`) — fixture and assertion updated to shorthand as part of TSK-261 (BUG-257 fix)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-20: `override_session_model_to_opus()` fires for shorthand `"sonnet"` input, writes `"opus"` (BUG-257 MRE)

- **Given:** `~/.claude/` directory exists. `~/.claude/settings.json` contains `{"model": "sonnet"}` (Claude Code shorthand alias).
- **When:** `override_session_model_to_opus(&paths)` is called directly.
- **Then:** Returns `true`. `~/.claude/settings.json` now contains `"model": "opus"`. Additional scenarios verified in the same test: full-ID input `"claude-sonnet-4-6"` also returns `true` and writes `"opus"`; absent model (empty settings.json) returns `true` and writes `"opus"`; non-Sonnet `"opus"` returns `false`, settings.json unchanged; non-Sonnet `"haiku"` returns `false`, settings.json unchanged; full-ID `"claude-opus-4-6"` returns `true` and writes `"opus"` shorthand — not full ID (Fix(BUG-286)).
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug257_override_shorthand_alias` (in `claude_profile_core/tests/account_test.rs`) — ✅ TSK-261
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-22: `seven_day_sonnet = None` — model override does not fire (BUG-300 MRE)

- **Given (unit test):** `apply_model_override` called with quota data where `seven_day_sonnet = None` (absent tier):
  - `~/.claude/settings.json` contains `"model": "claude-sonnet-4-6"` (Sonnet — the override would normally fire)
  - `ClaudePaths` pointing to a temp directory
- **When:** `apply_model_override(&data, &paths, false, "account.use", "alice@home.com")` called with `seven_day_sonnet = None`.
- **Then:**
  - `~/.claude/settings.json` is unchanged (still contains `"claude-sonnet-4-6"`) — override did not fire.
  - Second scenario (regression guard): same setup with `seven_day_sonnet = Some(PeriodUsage { utilization: 90.0, ... })` (10% left) — settings.json updated to `"claude-opus-4-6"`. Confirms `Some` path still fires correctly.
- **Exit:** n/a (unit test)
- **Note:** Fix(BUG-300): `map_or(0.0, ...)` on `seven_day_sonnet` returned `0.0` for `None`, which satisfies `< 20.0`, causing unconditional Opus override. `None` means tier absent (unknown), not exhausted. Fix: `if let Some(ref sonnet) = quota.seven_day_sonnet { if 100.0 - sonnet.utilization < 20.0 { ... } }`.
- **Source fn:** `mre_bug300_model_override_absent_sonnet_no_override` (in `src/usage/api_tests.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-21: Post-subprocess quota re-fetch updates in-memory quota; failure preserves pre-subprocess data (BUG-288 MRE)

- **Given:** `apply_post_switch_touch` is called with a valid `TouchCtx` containing quota data with `five_hour.resets_at = None` (idle). The subprocess runs and returns. A re-fetch is attempted via `fetch_oauth_usage`.
- **When (success path):** Re-fetch returns `Ok(new_data)` where `new_data.five_hour.resets_at = Some(...)` (active).
- **Then (success path):** The in-memory quota result reflects the post-subprocess state (`resets_at = Some`). A subsequent `apply_touch` call for the same account will see `all_running = true` and emit `skipped (reason: already active)` — no second subprocess spawned. Fix(BUG-288).
- **When (failure path):** Re-fetch returns `Err(...)`.
- **Then (failure path):** Pre-subprocess quota data is preserved; function returns without panicking or aborting the switch. The re-fetch failure is non-aborting.
- **Exit:** n/a (unit test — no exit code)
- **Source fn:** `mre_bug288_post_switch_touch_refetch_updates_quota` (structural + no-token failure path) + `it_apply_post_switch_touch_cred_file_absent_skips_refetch` (file-absent failure path) — both in `src/usage/api_tests.rs`
- **Source:** [feature/027_account_use_post_switch_touch.md AC-21](../../../docs/feature/027_account_use_post_switch_touch.md)
