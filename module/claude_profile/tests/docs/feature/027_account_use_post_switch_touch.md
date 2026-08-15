# Test: Feature 027 — `.account.use` Post-Switch Touch

### Scope

- **Purpose**: Test cases for post-switch touch after `.account.use`.
- **Source**: `docs/feature/027_account_use_post_switch_touch.md`
- **Covers**: AC-01 through AC-21

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
| FT-14 | `trace::1 touch::0` — no timestamped `account.use` diagnostic lines emitted | AC-15 | Integration |
| FT-15 | `trace::0` (default) — no timestamped `account.use` diagnostic lines emitted | AC-15 | Integration |
| FT-16 | `trace::` with bad value exits 1 | AC-16 | Integration |
| FT-17 | `touch::1` + fetch Err + expired `expiresAt` + `refresh::1` → refresh fails → exits 3; switch NOT executed | AC-17 | Integration (BUG-213 + BUG-230 MRE) |
| FT-18 | `touch::1` + fetch Err + expired `expiresAt` + `refresh::0` → exits 3 immediately; no refresh attempt | AC-20 | Integration (BUG-230) |
| FT-19 | Active account + 7d(Son) < 10% → model override fires after switch | AC-18 | Unit (BUG-238 MRE) |
| FT-20 | `override_session_model_to_opus()` fires for shorthand `"sonnet"` input, writes `"opus"` | AC-18 | Unit (BUG-257 MRE) |
| FT-21 | Post-subprocess re-fetch updates in-memory quota; failure preserves pre-subprocess data | AC-21 | Unit (BUG-288 MRE) |
| FT-22 | `seven_day_sonnet = None` → override fires conservatively; writes "sonnet" (Fix BUG-311) | AC-18 | Unit (BUG-300 + BUG-311 MRE) |
| FT-23 | model restored to sonnet when settings.json has "opus" and Sonnet quota sufficient (BUG-311 MRE) | AC-18 | Unit (BUG-311 MRE) |
| FT-24 | `trace::1` + model override fires → `model override: opus→sonnet` trace line emitted | AC-19 | Unit (BUG-311) |
| FT-25 | Structural — both trace format strings interpolate `{label}` verbatim | AC-19 | Structural |
| — | `trace::1` + model override fires → `model override: sonnet→opus` trace line emitted | AC-19 | Live-only (requires `trace::1` + `7d(Son) < 10%` + Sonnet model in snapshot) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-22 | seven_day_sonnet = None → override fires conservatively; writes "sonnet" (Fix BUG-311) | AC-18 | BUG-300 + BUG-311 MRE |
| FT-23 | model restored to "sonnet" when settings.json has "opus" and Sonnet quota sufficient | AC-18 | BUG-311 MRE |
| FT-24 | trace::1 + Sonnet quota sufficient → opus→sonnet trace line emitted | AC-19 | BUG-311 |
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
| FT-19 | active account + 7d(Son) < 10% → model override sonnet→opus after switch | AC-18 | BUG-238 MRE |
| FT-21 | post-subprocess re-fetch updates in-memory quota; failure preserves pre-subprocess data | AC-21 | BUG-288 MRE |
| FT-23 | model restored to sonnet when settings.json has "opus" and Sonnet quota sufficient | AC-18 | BUG-311 MRE |
| FT-24 | trace::1 with Sonnet quota sufficient → opus→sonnet trace line emitted | AC-19 | BUG-311 |
| FT-25 | structural — both trace format strings interpolate {label} verbatim | AC-19 | Structural |

**Total:** 25 FT cases

---

### FT-01: `touch::1` idle account dispatches subprocess after switch

- **Given:** Account `target@example.com` saved with a live OAuth token; `source@example.com` (same token) holds the per-machine active marker before the switch. Whether the 5h window is idle or active at test-run time depends on the live account's actual quota state — the test does not control this precondition (see Live annotation).
- **When:** `clp .account.use name::target@example.com touch::1`
- **Then:** Exits 0; stdout contains `switched` (generic substring — the test does not assert an account-name-specific message); credentials rotated. The subprocess dispatch itself is fire-and-forget and not directly asserted here — its trace-visible dispatch is verified by FT-11/FT-12, which cover the same live idle/active states with `trace::1`.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and idle `five_hour.resets_at = None` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-01](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-02: `touch::0` suppresses quota fetch and subprocess

- **Given:** Account `target@example.com` saved (no `accessToken` in the fixture — if touch were attempted, the quota fetch would fail; exit 0 proves touch was skipped before any quota API call, not merely that it happened to succeed).
- **When:** `clp .account.use name::target@example.com touch::0`
- **Then:** Exits 0; stdout contains `switched` (generic substring, no name-specific message asserted); no quota fetch performed; no subprocess dispatched. Behavior is identical to pre-Feature-027 `.account.use`.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` (in `account_relogin_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-02](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-03: Active account (`resets_at` present) — subprocess spawned idempotently (Fix BUG-285)

- **Given:** Account `target@example.com` saved with a live OAuth token (same underlying live-token fixture as FT-01 — `aw27` does not itself branch on or control `resets_at`; it only asserts the switch succeeds regardless of which live state is exercised).
- **When:** `clp .account.use name::target@example.com touch::1`
- **Then:** Exits 0; stdout contains `switched` (generic substring); credentials rotated. Fix(BUG-285): the idle check using `resets_at` as a subprocess identity oracle was removed; subprocess always fires when quota fetch succeeds, exiting immediately when the account is already active — this specific dispatch/exit mechanic is not directly asserted by `aw27` itself (see FT-12 for the trace-verified equivalent).
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and active `five_hour.resets_at` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-03](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-04: Quota fetch failure + `expiresAt` future — touch skipped silently, switch completes

- **Given:** Account `target@example.com` saved with a credential file that has no `accessToken` field and `expiresAt = FAR_FUTURE_MS` (far-future timestamp, not locally expired). Quota fetch against the saved credential file fails immediately (no `accessToken` → auth error). Because `expiresAt` is in the future, the expiry check passes — this is the non-expired path per AC-04. (See FT-17 for the expired-`expiresAt` path that exits 3.)
- **When:** `clp .account.use name::target@example.com` (default `touch::1`)
- **Then:** Exits 0; stdout contains `switched` (generic substring); touch skipped silently. Fetch failure with a non-expired `expiresAt` is non-fatal.
- **Exit:** 0
- **Source fn:** `aw23_touch_skipped_no_access_token` (in `account_relogin_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-04](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-05: `imodel::auto` delegates model selection to `resolve_model()`

- **Given:** Feature 026 unit tests cover `resolve_model()` exhaustively (FT-01 through FT-07 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — it calls `resolve_model(&quota, imodel_param)` with the quota fetched for the target account.
- **Then:** Model selection behavior is identical to `.usage` touch path — `imodel::auto` uses the `son_idle` gate (Haiku by default; Sonnet when `seven_day_sonnet` is present and `resets_at=None`). All resolution semantics are governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-01..FT-07)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_model` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-05](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-06: `effort::auto` delegates effort injection to `resolve_effort()`

- **Given:** Feature 026 unit tests cover `resolve_effort()` exhaustively (FT-08 through FT-12 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — `apply_post_switch_touch` (`src/usage/api_switch.rs`) calls `effort_pre_args(&model, effort_param)`, which internally wraps `resolve_effort()` and formats its result into subprocess CLI args.
- **Then:** Effort injection behavior is identical to `.usage` touch path — `effort::auto` injects `--effort low` for any model, nothing for `imodel::keep` or `imodel::haiku`. All injection semantics governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-08..FT-12)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_effort` is shared, invoked via `effort_pre_args`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-06](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-07: `imodel::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `aw24_imodel_bad_value_exits_1` (in `account_relogin_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-08: `effort::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `aw25_effort_bad_value_exits_1` (in `account_relogin_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-09: `dry::1` — no credentials modified, no subprocess spawned

- **Given:** Account `alice@home.com` saved (the sole account in the fixture — no second account or per-machine active marker is created; dry-run never reaches the quota-fetch/touch stage, so the account's 5h-window state is not exercised).
- **When:** `clp .account.use name::alice@home.com dry::1`
- **Then:** Exits 0; stdout contains `[dry-run] would switch to 'alice@home.com'` (asserted verbatim); credentials file unchanged (asserted via byte-identical before/after read). No subprocess dispatch is possible — the dry-run short-circuit fires before both credential rotation and the touch subprocess call site.
- **Exit:** 0
- **Source fn:** `aw02_switch_dry_run` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-08](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-10: `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` with correct defaults

- **Given:** Standard environment.
- **When:** `clp .account.use --help` (or `.account.use help::1`)
- **Then:** Exits 0. Help output contains `touch::` with default `1`, `refresh::` with default `1`, `imodel::` with default `auto`, `effort::` with default `auto`, and `trace::` with default `0`.
- **Exit:** 0
- **Source fn:** `aw26_help_shows_touch_imodel_effort` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09, AC-16](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-11: `trace::1 touch::1` account — subprocess always dispatched when quota fetch OK

- **Given:** Account `target@example.com` saved with a live OAuth token; `source@example.com` (same token) holds the active marker prior to the switch.
- **When:** `clp .account.use name::target@example.com trace::1`
- **Then:** Exits 0. Stdout contains `switched` (generic substring). Stderr unconditionally contains the `· account.use  ` trace prefix, `reading` + `reading: OK`, and never contains `idle check:` (Fix(BUG-285): idle-check trace line removed). The fetch-OK-only lines — `subprocess: scheduled (idle check removed)`, `model:`, `subprocess: spawned` — are asserted only when the live fetch actually succeeded (`if err.contains("quota fetch: OK")`); when the live fetch fails, the test logs a skip note instead of failing.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token)
- **Source fn:** `aw28_lim_it_trace_idle_account_all_lines` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-12: `trace::1 touch::1` account with active 5h window — subprocess spawned (idempotent)

- **Given:** Account `target@example.com` saved with a live OAuth token; `source@example.com` (same token) holds the active marker prior to the switch. Same live/uncontrolled `resets_at` caveat as FT-01/FT-03 — the test does not itself force an active window.
- **When:** `clp .account.use name::target@example.com trace::1`
- **Then:** Exits 0. Stderr unconditionally contains the `· account.use  ` trace prefix (no stdout assertion is made by this test at all). The fetch-OK-only lines — `subprocess: scheduled (idle check removed)`, `model:`, `effort:`, `subprocess: spawned`, and the absence of `subprocess: skipped (reason: already active)` — are asserted only when the live fetch actually succeeded (`if err.contains("quota fetch: OK")`); otherwise the test logs a skip note. Fix(BUG-285): `subprocess: skipped (reason: already active)` no longer emitted; subprocess is always dispatched and exits immediately when already active.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token with `five_hour.resets_at` present in live quota response)
- **Source fn:** `aw29_lim_it_trace_active_account_subprocess_skipped` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-13: `trace::1 touch::1` quota fetch failure + `expiresAt` in future — fetch-err + expiry-valid trace lines

- **Given:** Account `target@example.com` saved with an invalid `accessToken` (causes HTTP auth error) and `expiresAt = FAR_FUTURE_MS` (not expired). Quota fetch fails with an auth error. Because `expiresAt` is in the future, the expiry check passes and emits a `valid` trace line — the switch completes. (See FT-17 for the expired-`expiresAt` path that exits 3.)
- **When:** `clp .account.use name::target@example.com trace::1`
- **Then:** Exits 0. Stdout contains `switched` (generic substring). Stderr contains: `· account.use  ` prefix, `reading: OK`, `quota fetch: Err(`, `subprocess: skipped (reason: fetch failed)`, `expiry check: valid`. No `idle check:` or `model:` substrings appear.
- **Exit:** 0
- **Source fn:** `aw30_trace_fetch_failure_skips_idle_model_lines` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-14](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-14: `trace::1 touch::0` — no timestamped `account.use` diagnostic lines emitted

- **Given:** Account `target@example.com` saved (the account name is not material to this scenario — validation and the touch short-circuit both precede any name-specific behavior).
- **When:** `clp .account.use name::target@example.com touch::0 trace::1`
- **Then:** Exits 0. Stdout contains `switched` (generic substring). Stderr does not contain the `· account.use  ` trace prefix (no quota fetch operations performed).
- **Exit:** 0
- **Source fn:** `aw31_trace_touch_disabled_no_trace_lines` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-15: `trace::0` (default) — no timestamped `account.use` diagnostic lines emitted

- **Given:** Account `target@example.com` saved. Note: the cited test passes `touch::0` explicitly (the same fixture as FT-02), not a fully-default invocation with implicit `touch::1` — no test in this suite exercises a fully-default invocation with an explicit stderr assertion.
- **When:** `clp .account.use name::target@example.com touch::0` (default `trace::0`)
- **Then:** Exits 0. Stdout contains `switched` (generic substring). `aw22` itself makes no assertion on stderr — the "no diagnostic lines" property is not directly checked here; it is structurally guaranteed by every `account.use` trace line being written inside an `if trace { eprintln!(...) }` guard (confirmed via `src/commands/account_ops.rs`), so `trace::0` (the default, since `trace::1` is never passed) unconditionally suppresses all diagnostic output regardless of `touch::`.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` (in `account_relogin_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-16: `trace::` with bad value exits 1

- **Given:** Any account store state.
- **When:** `clp .account.use name::alice@home.com trace::bad`
- **Then:** Exits 1. Stderr names the four valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `aw32_trace_bad_value_exits_1` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-16](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-17: `touch::1` + fetch Err + expired `expiresAt` + `refresh::1` — refresh fails → exits 3 (BUG-213 + BUG-230 MRE)

- **Given:** Account `alice@home.com` saved with a credential file where `expiresAt` is set to a timestamp in the past (locally expired token) and no `accessToken` (so the refresh subprocess immediately fails). Default `refresh::1` applies.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1 refresh::1`)
- **Then:** Exits 3. Stderr contains `account credentials expired and refresh failed: alice@home.com (expired ...ago)`. `~/.claude/.credentials.json` is NOT overwritten. The active marker is NOT updated.
- **Exit:** 3
- **Source fn:** `mre_bug213_account_use_refuses_expired_token_on_fetch_error` + `mre_bug230_account_use_refresh_fails_exits_3_with_updated_message` (in `account_relogin_test_b.rs`)
- **Note:** BUG-213 MRE still passes — `err.contains("account credentials expired")` holds because the new message `"account credentials expired and refresh failed"` is a superset. The BUG-230 MRE additionally asserts `err.contains("and refresh failed")`. For `refresh::0` (immediate exit), see FT-18. The discriminant between FT-04 and FT-17 is the `expires_at_ms` argument to `write_account()`: `FAR_FUTURE_MS` (future, not expired) vs `1000` (past, expired).
- **Source:** [feature/027_account_use_post_switch_touch.md AC-17](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-18: `touch::1` + fetch Err + expired `expiresAt` + `refresh::0` — exits 3 immediately (BUG-230)

- **Given:** Account `alice@home.com` saved with a credential file where `expiresAt` is in the past and no `accessToken`. `refresh::0` explicitly disables the refresh attempt.
- **When:** `clp .account.use name::alice@home.com refresh::0 trace::1`
- **Then:** Exits 3. Stderr contains `account credentials expired: alice@home.com (expired ...ago)`. Does NOT contain `"and refresh failed"` (no refresh attempted). Trace contains `refused (refresh::0)`. `~/.claude/.credentials.json` is NOT overwritten.
- **Exit:** 3
- **Source fn:** `aw33_refresh_disabled_exits_3_immediately` (in `account_relogin_test_b.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-20](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-19: Active account + 7d(Son) < 10% — model override sonnet→opus fires after switch (BUG-238 MRE)

- **Given (unit test):** `apply_model_override` called directly (no CLI invocation) with `OauthUsageData { five_hour: None, seven_day: None, seven_day_sonnet: Some(PeriodUsage { utilization: 91.0, resets_at: None }) }` (9% Sonnet quota left) and an empty/absent `~/.claude/settings.json` (only the parent directory is pre-created; no `{"model": "sonnet"}` is pre-seeded).
- **When:** `apply_model_override(&quota, &paths, false, "account.use", "test-account", AccountBackend::Anthropic)` called directly.
- **Then:** `~/.claude/settings.json` is written with `"model": "opus"` (shorthand — not the full ID `"claude-opus-4-8"`). This proves the fix wiring `apply_model_override` into the (subsequently removed) `AlreadyActive` branch — originally the override was skipped for already-active accounts; the fix ensures it fires unconditionally whenever the quota fetch succeeds, active or idle. The account-name/`five_hour`/`{name}.json` framing in this AC is illustrative of the end-to-end `.account.use` scenario the fix targets — this specific test exercises `apply_model_override` in isolation, not through a live `.account.use` CLI invocation.
- **Exit:** n/a (unit test — no CLI invocation, no exit code)
- **Source fn:** `mre_bug238_model_override_fires_for_active_account` (in `tests/usage/api_tests_a.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-20: `override_session_model_to_opus()` fires for shorthand `"sonnet"` input, writes `"opus"` (BUG-257 MRE)

- **Given:** `~/.claude/` directory exists. `~/.claude/settings.json` contains `{"model": "sonnet"}` (Claude Code shorthand alias).
- **When:** `override_session_model_to_opus(&paths)` is called directly.
- **Then:** Returns `true`. `~/.claude/settings.json` now contains `"model": "opus"`. Additional scenarios verified in the same test: full-ID input `"claude-sonnet-4-6"` also returns `true` (regression guard, return value only — content not re-asserted); absent model (empty `{}` settings.json) returns `true` and writes `"opus"`; non-Sonnet `"opus"` returns `false`, settings.json unchanged; non-Sonnet `"haiku"` returns `false`, settings.json unchanged; full-ID `"claude-opus-4-6"` returns `true` and writes `"opus"` shorthand — not full ID (Fix(BUG-286)).
- **Exit:** n/a (unit test)
- **Source fn:** `mre_bug257_override_shorthand_alias` (in `claude_profile_core/tests/account_test.rs`) — ✅ TSK-261
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-22: `seven_day_sonnet = None` — override fires conservatively; writes "sonnet" (BUG-300 + BUG-311)

- **Given (unit test):** `apply_model_override` called with `OauthUsageData { five_hour: None, seven_day: None, seven_day_sonnet: None }` (absent tier). `~/.claude/settings.json` does not exist (only its parent directory is pre-created) — no pre-seeded `"claude-sonnet-5"` or any other model value.
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account", AccountBackend::Anthropic)` called with `seven_day_sonnet = None`.
- **Then:** `~/.claude/settings.json` contains `"model": "sonnet"` — written fresh into the previously-absent file (not a normalization of a pre-existing full-ID value; that normalization scenario is covered separately by FT-20's full-ID sonnet case). `"opus"` does NOT appear. The absent-tier path calls `override_session_model_to_sonnet()` conservatively (Fix BUG-311).
- **Exit:** n/a (unit test)
- **Note (BUG-300):** `map_or(0.0, ...)` caused `None` to fire unconditional Opus override. Fixed by `if let Some(ref sonnet)` guard. **(BUG-311):** the `else` (tier-absent) now conservatively calls `override_session_model_to_sonnet()` — absent tier means unknown, not exhausted. This single-scenario test does not itself include a second `Some`+exhausted regression case in the same function — that path is covered separately by FT-19 (`mre_bug238`) and FT-23 (`mre_bug311`).
- **Source fn:** `mre_bug300_model_override_absent_sonnet_no_override` (in `tests/usage/api_tests_a.rs`) — assertion updated post-BUG-311 to check "sonnet" written, "opus" absent.
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-21: Post-subprocess quota re-fetch updates in-memory quota; failure preserves pre-subprocess data (BUG-288 MRE)

- **Given:** Two independent checks, both against `apply_post_switch_touch` with `TouchCtx::for_test(OauthUsageData { five_hour: None, seven_day: None, seven_day_sonnet: None })`:
  1. **Structural (success-path wiring):** the source of `apply_post_switch_touch` in `src/usage/api_switch.rs` is inspected via `include_str!` and must contain both `fetch_oauth_usage` and `write_quota_cache` — this proves the re-fetch-and-cache-update call sites exist in source; it does not execute a successful re-fetch at runtime.
  2. **Runtime (two failure-mode guards, neither reaching an actual `fetch_oauth_usage` call):** (a) `mre_bug288_...`'s runtime section writes a `{name}.credentials.json` with no `accessToken` field — the inner `parse_string_field` guard fails, so the re-fetch is skipped before any HTTP attempt (not "returns `Err`"); (b) `it_apply_post_switch_touch_cred_file_absent_skips_refetch` covers the outer guard — no credentials file at all, so `read_to_string` fails and the entire re-fetch block is bypassed.
- **When:** `apply_post_switch_touch(name, ctx, "auto", "auto", false, &paths, paths.base())` called directly (not via CLI) for each runtime scenario.
- **Then:** Both runtime scenarios: no panic; `last_touch_at` and `touch_idle` are still written to the `{name}.json` cache (written unconditionally, before the re-fetch block); `resets_at` does NOT appear in the cache (re-fetch never wrote new quota data). The success-path claim — in-memory quota reflecting `resets_at = Some(...)` after a genuinely successful re-fetch, and a subsequent `apply_touch` call seeing `all_running = true` — is confirmed only structurally (the calls exist in source); no test in this pair exercises a successful re-fetch end-to-end.
- **Exit:** n/a (unit test — no exit code)
- **Source fn:** `mre_bug288_post_switch_touch_refetch_updates_quota` (structural + no-token failure path) + `it_apply_post_switch_touch_cred_file_absent_skips_refetch` (file-absent failure path) — both in `tests/usage/api_tests_b.rs`
- **Source:** [feature/027_account_use_post_switch_touch.md AC-21](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-23: Model restored to "sonnet" when settings.json has "opus" and Sonnet quota sufficient (BUG-311 MRE)

- **Given (unit test):** `apply_model_override` called with quota data where `seven_day_sonnet = Some(PeriodUsage { utilization: 4.0 })` (96% left — well above 10% threshold). `~/.claude/settings.json` pre-seeded with `"model": "opus"` (stale from previous exhaustion cycle).
- **When:** `apply_model_override(&quota, &paths, false, "usage", "test-account", AccountBackend::Anthropic)` called directly (label `"usage"`, not `"account.use"` — this test exercises the shared `apply_model_override` function independent of caller).
- **Then:** `~/.claude/settings.json` contains `"model": "sonnet"` and no longer contains `"opus"` — asserted as a single combined condition on file content. The test does not capture or assert any boolean return value from `apply_model_override`/`override_session_model_to_sonnet()`.
- **Exit:** n/a (unit test)
- **Note:** Reproduces the user-visible symptom of BUG-311: after `.account.use` switches to an account with plenty of Sonnet quota, the `.usage` footer still showed `opus` because `apply_model_override()` had no else-branch to restore `"sonnet"`.
- **Source fn:** `mre_bug311_model_restored_to_sonnet_when_opus_and_quota_sufficient` (in `tests/usage/api_tests_a.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-18](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-24: `trace::1` with Sonnet quota sufficient → `opus→sonnet` trace line emitted (BUG-311)

- **Given (unit test):** `OauthUsageData` with `seven_day_sonnet = Some(PeriodUsage { utilization: 4.0 })` (96% left — well above the 10% threshold).
- **When:** `model_override_direction(&quota)` is called.
- **Then:** Returns `Some("opus→sonnet")` — the oracle's direction matches the direction word `apply_model_override()`'s trace format string embeds verbatim when `trace=true` (see FT-25, which structurally confirms the format string interpolates this direction).
- **Exit:** n/a (unit test)
- **Source fn:** `t09_model_override_trace_opus_to_sonnet` (in `tests/usage/api_tests_a.rs`)
- **Note:** Converted from gag-based stderr capture to a direct `model_override_direction()` oracle call. The actual file-write behavior for this scenario remains covered by the untouched sibling test `mre_bug311_model_restored_to_sonnet_when_opus_and_quota_sufficient` (FT-23).
- **Source:** [feature/027_account_use_post_switch_touch.md AC-19](../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-25: Structural — `apply_model_override`'s trace format strings interpolate `{label}` verbatim

- **Given (structural test):** Source of `src/usage/api_switch.rs`.
- **When:** The source is inspected via `include_str!`.
- **Then:** Both the `sonnet→opus` and `opus→sonnet` trace format strings contain the literal `{label}` interpolation (not a hardcoded label), so any caller-supplied label (e.g. `"usage"`, `"account.use"`) appears verbatim in the trace.
- **Exit:** n/a (unit test)
- **Source fn:** `t08_model_override_trace_label_is_usage` (in `tests/usage/api_tests_a.rs`)
- **Note:** Converted from gag-based stderr capture (asserting `" · usage"` appeared and `" · account.use  "` did not in captured output for one label value) to a structural check that both format strings embed `{label}` — a stronger, label-value-independent proof of the same property.
- **Source:** [feature/027_account_use_post_switch_touch.md AC-19](../../../docs/feature/027_account_use_post_switch_touch.md)
