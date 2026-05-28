# Test: Feature 027 — `.account.use` Post-Switch Touch

Feature behavioral requirement test cases for `docs/feature/027_account_use_post_switch_touch.md`. Each FT case maps to one acceptance criterion. Command-level integration tests (IT-N) are in [cli/command/005_account_use.md](../cli/command/005_account_use.md) (IT-17 through IT-23). Model/effort resolution unit tests are in [feature/026_subprocess_model_effort.md](026_subprocess_model_effort.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::1` idle account → subprocess dispatched after switch | AC-01 | Live |
| FT-02 | `touch::0` idle account → pure rotation, no subprocess | AC-02 | Integration |
| FT-03 | Active account (`resets_at` present) → no subprocess | AC-03 | Live |
| FT-04 | Quota fetch failure + token NOT expired → touch skipped, switch completes, exits 0 | AC-04 | Integration |
| FT-05 | `imodel::auto` model selection delegates to `resolve_model()` | AC-05 | Structural (→ Feature 026) |
| FT-06 | `effort::auto` effort injection delegates to `resolve_effort()` | AC-06 | Structural (→ Feature 026) |
| FT-07 | `imodel::bad` exits 1 naming all five valid values | AC-07 | Integration |
| FT-08 | `effort::bad` exits 1 naming all five valid values | AC-07 | Integration |
| FT-09 | `dry::1` — no credentials modified, no subprocess spawned | AC-08 | Integration |
| FT-10 | `touch::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` with defaults | AC-09, AC-16 | Integration |
| FT-11 | `trace::1 touch::1` idle account — all 6 trace lines emitted in order | AC-10, AC-11, AC-12, AC-13, AC-14 | Integration |
| FT-12 | `trace::1 touch::1` active account — read+fetch+idle-check+model+subprocess-skipped lines | AC-10, AC-11, AC-12, AC-13, AC-14 | Integration |
| FT-13 | `trace::1 touch::1` fetch failure — read+fetch-err emitted; idle/model/subprocess omitted | AC-10, AC-11, AC-14 | Integration |
| FT-14 | `trace::1 touch::0` — no `[trace] account.use` lines emitted | AC-15 | Integration |
| FT-15 | `trace::0` (default) — no `[trace] account.use` lines emitted | AC-15 | Integration |
| FT-16 | `trace::` with bad value exits 1 | AC-16 | Integration |
| FT-17 | `touch::1` + fetch Err + expired `expiresAt` → exits 3; switch NOT executed | AC-17 | Integration (BUG-213 MRE) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::1 idle account dispatches subprocess | AC-01 | Touch Dispatch |
| FT-02 | touch::0 suppresses subprocess and quota fetch | AC-02 | Touch Suppression |
| FT-03 | active account suppresses subprocess | AC-03 | Idle Guard |
| FT-04 | fetch failure + not expired — skip silently, exits 0 | AC-04 | Graceful Degradation |
| FT-05 | imodel::auto delegates to resolve_model() | AC-05 | Model Delegation |
| FT-06 | effort::auto delegates to resolve_effort() | AC-06 | Effort Delegation |
| FT-07 | imodel::bad exits 1 with valid values | AC-07 | Rejection |
| FT-08 | effort::bad exits 1 with valid values | AC-07 | Rejection |
| FT-09 | dry::1 performs no modification | AC-08 | Dry Run |
| FT-10 | touch:: imodel:: effort:: trace:: in help with defaults | AC-09, AC-16 | Help Output |
| FT-11 | trace::1 touch::1 idle account — all 6 trace lines emitted | AC-10, AC-11, AC-12, AC-13, AC-14 | Trace Output |
| FT-12 | trace::1 touch::1 active account — read+fetch+idle-check+model+skipped lines | AC-10, AC-11, AC-12, AC-13, AC-14 | Trace Output |
| FT-13 | trace::1 touch::1 fetch failure — read+fetch-err emitted; idle/model/subprocess omitted | AC-10, AC-11, AC-14 | Trace Output |
| FT-14 | trace::1 touch::0 — no trace lines emitted | AC-15 | Trace Suppression |
| FT-15 | trace::0 (default) — no trace lines emitted | AC-15 | Trace Default |
| FT-16 | trace:: in .account.use --help with default 0 | AC-16 | Help Output |
| FT-17 | touch::1 + fetch Err + expired expiresAt → exits 3; no switch_account call | AC-17 | BUG-213 MRE |

**Total:** 17 FT cases

---

### FT-01: `touch::1` idle account dispatches subprocess after switch

- **Given:** Account `alice@home.com` saved with valid OAuth token and idle 5h window (`five_hour.resets_at` is absent). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; an isolated subprocess (`run_isolated`) is dispatched to start the idle 5h session window.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and idle `five_hour.resets_at = None` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-01](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-02: `touch::0` suppresses quota fetch and subprocess

- **Given:** Account `alice@home.com` saved with idle 5h window.
- **When:** `clp .account.use name::alice@home.com touch::0`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; no quota fetch performed; no subprocess dispatched. Behavior is identical to pre-Feature-027 `.account.use`.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-02](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-03: Active account (`resets_at` present) — no subprocess spawned

- **Given:** Account `alice@home.com` saved with valid OAuth token and an active 5h window (`five_hour.resets_at` is set).
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; no subprocess dispatched (account already has an active session).
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and active `five_hour.resets_at` in live quota response)
- **Source fn:** `aw27_lim_it_touch_with_live_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-03](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-04: Quota fetch failure + `expiresAt` absent — touch skipped silently, switch completes

- **Given:** Account `alice@home.com` saved with a credential file that has no `accessToken` field and no `expiresAt` field. Quota fetch against the saved credential file will fail with an auth error. Because `expiresAt` is absent, the expiry check is skipped — this is the non-expired path per AC-04. (See FT-17 for the expired-token case.)
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; touch skipped silently. No error output. Fetch failure with absent `expiresAt` is non-fatal.
- **Exit:** 0
- **Source fn:** `aw23_touch_skipped_no_access_token` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-04](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-05: `imodel::auto` delegates model selection to `resolve_model()`

- **Given:** Feature 026 unit tests cover `resolve_model()` exhaustively (FT-01 through FT-07 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — it calls `resolve_model(&quota, imodel_param)` with the quota fetched for the target account.
- **Then:** Model selection behavior is identical to `.usage` touch path — `imodel::auto` picks Sonnet when `7d(Son) ≥ 30%`, Opus otherwise. All resolution semantics are governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-01..FT-07)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_model` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-05](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-06: `effort::auto` delegates effort injection to `resolve_effort()`

- **Given:** Feature 026 unit tests cover `resolve_effort()` exhaustively (FT-08 through FT-12 in [026_subprocess_model_effort.md](026_subprocess_model_effort.md)).
- **When:** `.account.use` dispatches its post-switch touch subprocess — it calls `resolve_effort(&model, effort_param)` with the resolved model.
- **Then:** Effort injection behavior is identical to `.usage` touch path — `effort::auto` injects `--effort high` for Sonnet, `--effort max` for Opus, nothing for `imodel::keep` or `imodel::haiku`. All injection semantics governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-08..FT-12)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_effort` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-06](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-07: `imodel::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `aw24_imodel_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-08: `effort::bad` exits 1 naming all five valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `aw25_effort_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-09: `dry::1` — no credentials modified, no subprocess spawned

- **Given:** Account `alice@home.com` saved with idle 5h window. Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com dry::1`
- **Then:** Exits 0; stdout contains `[dry-run] would switch to 'alice@home.com'`; credentials file unchanged; active marker unchanged; no subprocess dispatched. The dry-run short-circuit fires before both credential rotation and touch subprocess.
- **Exit:** 0
- **Source fn:** `aw02_switch_dry_run` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-08](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-10: `touch::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` with correct defaults

- **Given:** Standard environment.
- **When:** `clp .account.use --help` (or `.account.use help::1`)
- **Then:** Exits 0. Help output contains `touch::` with default `1`, `imodel::` with default `auto`, `effort::` with default `auto`, and `trace::` with default `0`.
- **Exit:** 0
- **Source fn:** `aw26_help_shows_touch_imodel_effort` (in `tests/cli/account_mutations_test.rs`) — `trace::` presence asserted; default `0` value not yet verified
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09, AC-16](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-11: `trace::1 touch::1` idle account — all 6 trace lines emitted

- **Given:** Account `alice@home.com` saved with valid OAuth token and idle 5h window (`five_hour.resets_at` absent). Credential store has a valid `alice@home.com.credentials.json`.
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr (in order): `[trace] account.use  alice@home.com  reading {path}`, `reading: OK`, `quota fetch: OK`, `idle check: resets_at=absent → idle`, `model: {model}  effort: {effort}`, `subprocess: spawned`.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and `five_hour.resets_at = None` in live quota response)
- **Source fn:** `aw28_lim_it_trace_idle_account_all_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-12: `trace::1 touch::1` active account — reading + fetch + idle-check + model + subprocess-skipped

- **Given:** Account `alice@home.com` saved with valid OAuth token and an active 5h window (`five_hour.resets_at` present).
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr (in order): `reading {path}`, `reading: OK`, `quota fetch: OK`, `idle check: resets_at=present → already active`, `model: {model}  effort: {effort}`, `subprocess: skipped (reason: already active)`. The model/effort line IS emitted — quota fetch succeeded so model/effort can be resolved regardless of idle state.
- **Exit:** 0
- **Live:** yes (requires valid OAuth token and `five_hour.resets_at` present in live quota response)
- **Source fn:** `aw29_lim_it_trace_active_account_subprocess_skipped` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-12, AC-13, AC-14](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-13: `trace::1 touch::1` quota fetch failure + `expiresAt` absent — read + fetch-err lines only

- **Given:** Account `alice@home.com` saved with a credential file that has no `accessToken` and no `expiresAt` field. Quota fetch will fail with an auth error. Because `expiresAt` is absent, no `expiry check:` trace line is emitted — the expiry check is skipped. (See FT-17 for the expired-`expiresAt` trace path.)
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr contains (in order): `reading {path}`, `reading: OK`, `quota fetch: Err({msg})`, `subprocess: skipped (reason: fetch failed)`. No `idle check:`, `model:`, or `expiry check:` lines emitted.
- **Exit:** 0
- **Source fn:** `aw30_trace_fetch_failure_skips_idle_model_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10, AC-11, AC-14](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-14: `trace::1 touch::0` — no `[trace] account.use` lines emitted

- **Given:** Any account store state.
- **When:** `clp .account.use name::alice@home.com touch::0 trace::1`
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr: no `[trace] account.use` lines (no quota fetch operations performed).
- **Exit:** 0
- **Source fn:** `aw31_trace_touch_disabled_no_trace_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-15: `trace::0` (default) — no `[trace] account.use` lines emitted

- **Given:** Account `alice@home.com` saved with valid credentials and idle 5h window.
- **When:** `clp .account.use name::alice@home.com` (default `trace::0`, default `touch::1`)
- **Then:** Exits 0. Stdout: `switched to 'alice@home.com'`. Stderr: no `[trace] account.use` lines. This is the standard non-diagnostic output path.
- **Exit:** 0
- **Source fn:** `aw22_touch_disabled_switch_succeeds` — already covers absence of trace output for non-trace invocations
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-16: `trace::` with bad value exits 1

- **Given:** Any account store state.
- **When:** `clp .account.use name::alice@home.com trace::bad`
- **Then:** Exits 1. Stderr names the four valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `aw32_trace_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-16](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-17: `touch::1` + fetch Err + expired `expiresAt` — exits 3, switch NOT executed (BUG-213 MRE)

- **Given:** Account `alice@home.com` saved with a credential file where `expiresAt` is set to a timestamp in the past (locally expired token). The quota fetch against that token returns an Err (e.g., HTTP 429 or auth error).
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 3. Stderr contains `account credentials expired: alice@home.com (expired ...ago)`. `~/.claude/.credentials.json` is NOT overwritten — `switch_account()` was not called. The active marker (`_active_{hostname}_{user}`) is NOT updated.
- **Exit:** 3
- **Source fn:** `⏳ mre_bug213_account_use_refuses_expired_token_on_fetch_error` (in `tests/cli/account_mutations_test.rs`)
- **Note:** BUG-213 MRE — verifies that a locally-expired token causes `.account.use` to refuse the switch rather than silently installing unusable credentials. The complementary `aw23_touch_skipped_no_access_token` (AC-04) must still pass — it uses a credential file with no `accessToken` (and thus no parseable `expiresAt`), which is treated as "not expired" and falls through to the silent-skip path.
- **Source:** [feature/027_account_use_post_switch_touch.md AC-17](../../../../docs/feature/027_account_use_post_switch_touch.md)
