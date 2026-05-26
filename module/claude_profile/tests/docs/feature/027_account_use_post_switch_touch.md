# Test: Feature 027 — `.account.use` Post-Switch Touch

Feature behavioral requirement test cases for `docs/feature/027_account_use_post_switch_touch.md`. Each FT case maps to one acceptance criterion. Command-level integration tests (IT-N) are in [cli/command/005_account_use.md](../cli/command/005_account_use.md) (IT-17 through IT-23). Model/effort resolution unit tests are in [feature/026_subprocess_model_effort.md](026_subprocess_model_effort.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::1` idle account → subprocess dispatched after switch | AC-01 | Live |
| FT-02 | `touch::0` idle account → pure rotation, no subprocess | AC-02 | Integration |
| FT-03 | Active account (`resets_at` present) → no subprocess | AC-03 | Live |
| FT-04 | Quota fetch failure → touch skipped, switch completes, exits 0 | AC-04 | Integration |
| FT-05 | `imodel::auto` model selection delegates to `resolve_model()` | AC-05 | Structural (→ Feature 026) |
| FT-06 | `effort::auto` effort injection delegates to `resolve_effort()` | AC-06 | Structural (→ Feature 026) |
| FT-07 | `imodel::bad` exits 1 naming all four valid values | AC-07 | Integration |
| FT-08 | `effort::bad` exits 1 naming all three valid values | AC-07 | Integration |
| FT-09 | `dry::1` — no credentials modified, no subprocess spawned | AC-08 | Integration |
| FT-10 | `touch::`, `imodel::`, `effort::` appear in `.account.use --help` with defaults | AC-09 | Integration |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::1 idle account dispatches subprocess | AC-01 | Touch Dispatch |
| FT-02 | touch::0 suppresses subprocess and quota fetch | AC-02 | Touch Suppression |
| FT-03 | active account suppresses subprocess | AC-03 | Idle Guard |
| FT-04 | fetch failure — skip silently, exits 0 | AC-04 | Graceful Degradation |
| FT-05 | imodel::auto delegates to resolve_model() | AC-05 | Model Delegation |
| FT-06 | effort::auto delegates to resolve_effort() | AC-06 | Effort Delegation |
| FT-07 | imodel::bad exits 1 with valid values | AC-07 | Rejection |
| FT-08 | effort::bad exits 1 with valid values | AC-07 | Rejection |
| FT-09 | dry::1 performs no modification | AC-08 | Dry Run |
| FT-10 | touch:: imodel:: effort:: in help with defaults | AC-09 | Help Output |

**Total:** 10 FT cases

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

### FT-04: Quota fetch failure — touch skipped silently, switch completes

- **Given:** Account `alice@home.com` saved with an invalid/expired `accessToken`. Quota fetch against the saved credential file will fail with an auth error.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; touch skipped silently. No error output. Fetch failure is non-fatal.
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
- **Then:** Effort injection behavior is identical to `.usage` touch path — `effort::auto` injects `--effort high` for Sonnet, `--effort max` for Opus, nothing for `imodel::keep`. All injection semantics governed by Feature 026.
- **Exit:** n/a (structural — no new unit test; coverage via Feature 026 FT-08..FT-12)
- **Source fn:** (covered by Feature 026 unit tests — `resolve_effort` is shared)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-06](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-07: `imodel::bad` exits 1 naming all four valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1. Stderr contains each of the four valid values: `auto`, `sonnet`, `opus`, `keep`.
- **Exit:** 1
- **Source fn:** `aw24_imodel_bad_value_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)

---

### FT-08: `effort::bad` exits 1 naming all three valid values

- **Given:** Any account store state (empty store is sufficient — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1. Stderr contains each of the three valid values: `auto`, `high`, `max`.
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

### FT-10: `touch::`, `imodel::`, `effort::` appear in `.account.use --help` with correct defaults

- **Given:** Standard environment.
- **When:** `clp .account.use --help` (or `.account.use help::1`)
- **Then:** Exits 0. Help output contains `touch::` with default `1`, `imodel::` with default `auto`, and `effort::` with default `auto`.
- **Exit:** 0
- **Source fn:** `aw26_help_shows_touch_imodel_effort` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09](../../../../docs/feature/027_account_use_post_switch_touch.md)
