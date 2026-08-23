# Test: Feature 030 — Account Billing Renewal Override

### Scope

- **Purpose**: Test cases for billing renewal date override, clearing, auto-advance, and `~Renews` rendering.
- **Source**: `docs/feature/030_account_renewal_override.md`
- **Covers**: AC-01 through AC-15

Feature behavioral requirement test cases for `docs/feature/030_account_renewal_override.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/014_account_renewal.md](../cli/command/14_account_renewal.md).

### AC Coverage Index

| FT | Criterion | AC | Source |
|----|-----------|-----|--------|
| FT-01 | `at::` writes `_renewal_at` preserving `oauthAccount` | AC-01 | account_renewal_test.rs |
| FT-02 | `from_now::+1h30m` writes future ISO-8601 UTC | AC-02 | account_renewal_test.rs |
| FT-03 | `from_now::-30m` writes past; `.usage` auto-advances | AC-03 | account_renewal_test.rs |
| FT-04 | `clear::1` removes `_renewal_at`; `oauthAccount` preserved | AC-04 | account_renewal_test.rs |
| FT-05 | `name::all from_now::+0m` updates all accounts | AC-05 | account_renewal_test.rs |
| FT-06 | `dry::1` prints value without writing | AC-06 | account_renewal_test.rs |
| FT-07 | `at::` + `from_now::` together exits 1 | AC-07 | account_renewal_test.rs |
| FT-08 | `at::` + `clear::` together exits 1 | AC-08 | account_renewal_test.rs |
| FT-09 | `from_now::` + `clear::` together exits 1 | AC-09 | account_renewal_test.rs |
| FT-10 | Past `_renewal_at` auto-advanced monthly at render | AC-10 | usage_model_test.rs |
| FT-11 | `~Renews` exact/estimated/absent rendering | AC-11 | format_tests.rs / unit |
| FT-12 | No operation param exits 1 | AC-12 | account_renewal_test.rs |
| FT-13 | Unknown account exits 2 | AC-13 | account_renewal_test.rs |
| FT-14 | Comma-list updates two accounts | AC-14 | account_renewal_test.rs |
| FT-15 | Partial comma-list: unknown account reported; others proceed | AC-15 | account_renewal_test.rs |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `at::` writes `_renewal_at`; `oauthAccount` unchanged | AC-01 | Write |
| FT-02 | `from_now::+1h30m` produces future timestamp | AC-02 | Write |
| FT-03 | `from_now::-30m` past value; auto-advance in `.usage` | AC-03 | Write + Render |
| FT-04 | `clear::1` removes key; `oauthAccount` preserved | AC-04 | Remove |
| FT-05 | `name::all from_now::+0m` updates all accounts | AC-05 | Bulk |
| FT-06 | `dry::1` no file write | AC-06 | Dry Run |
| FT-07 | `at::` + `from_now::` exits 1 | AC-07 | Conflict |
| FT-08 | `at::` + `clear::` exits 1 | AC-08 | Conflict |
| FT-09 | `from_now::` + `clear::` exits 1 | AC-09 | Conflict |
| FT-10 | Past `_renewal_at` auto-advanced monthly | AC-10 | Render |
| FT-11 | `~Renews` renders exact/estimated/absent correctly | AC-11 | Render |
| FT-12 | No operation param exits 1 | AC-12 | Validation |
| FT-13 | Unknown account exits 2 | AC-13 | Not Found |
| FT-14 | Comma-list updates two accounts | AC-14 | Bulk |
| FT-15 | Partial comma-list: unknown reported; others succeed | AC-15 | Bulk Partial |

**Total:** 15 FT cases

---

### FT-01: `at::` writes `_renewal_at`; `oauthAccount` content preserved

- **Given:** Account `test@example.com` exists with a full `oauthAccount` object in `{name}.json`.
- **When:** `clp .account.renewal name::test@example.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` contains `"_renewal_at": "2026-06-29T21:00:00Z"`. Existing `oauthAccount` subtree is unchanged (read-merge preserved it).
- **Exit:** 0
- **Source fn:** `ft01_account_renewal_at_writes_renewal_at` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-01](../../../docs/feature/030_account_renewal_override.md)

---

### FT-02: `from_now::+1h30m` produces future ISO-8601 UTC timestamp

- **Given:** Account `test@example.com` exists (credential only — no pre-existing `{name}.json`).
- **When:** `clp .account.renewal name::test@example.com from_now::+1h30m`
- **Then:** Exits 0. `_renewal_at` is written as an ISO-8601 UTC string starting with `"202"` (not `"200"`).
- **Exit:** 0
- **Note:** The cited test performs only a coarse year-prefix sanity check (`starts_with "202"`, `not "200"`) — it does not assert the precise `±10s` window around `now+5400s` (1h30m) this FT case originally claimed, nor does it assert `oauthAccount` preservation (the Given here never creates an `oauthAccount` subtree to preserve, unlike FT-01/FT-04). The `h`/`m` unit-conversion math itself is confirmed correct by direct inspection of `parse_from_now_delta()` in `claude_profile_core/src/account/renewal.rs`, but no test in this workspace independently re-derives and checks the precise resulting timestamp.
- **Source fn:** `ft02_account_renewal_from_now_positive` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-02](../../../docs/feature/030_account_renewal_override.md)

---

### FT-03: `from_now::-30m` past value stored; `.usage` auto-advances monthly

- **Given:** Account `test@example.com` exists (write step); separately, `past@renewal.com` with `_renewal_at: "2020-03-15T00:00:00Z"` (render step — a different, independent test/account).
- **When:** `clp .account.renewal name::test@example.com from_now::-30m`; then, separately, `clp .usage` against an account with a deeply-past `_renewal_at`.
- **Then (write):** Exits 0. `_renewal_at` is written as an ISO-8601 UTC string starting with `"202"` (not `"2099"`).
- **Then (render):** `.usage` auto-advances the past timestamp; `~Renews` column shows `in Xd` (exact, no `~` prefix) rather than an estimate.
- **Exit:** 0 (both steps)
- **Note:** The write step's cited test only performs a coarse year-prefix check (`starts_with "202"`, `not "2099"`) — it does not independently verify the value is exactly `~30m` in the past. The render step's cited test uses a separate account/fixture than the write step (no shared state) and only checks the `in `/no-`~` prefix — it does not assert the resulting day-count falls within 30 days. Neither step requires live credentials: both use `write_account()` (no token), and `it151` confirms in its own doc comment "No live credentials needed."
- **Source fn:** `ft03_account_renewal_from_now_negative` (write, in `tests/cli/account_renewal_test.rs`); `it151_past_renewal_at_auto_advances_in_usage` (render, in `tests/cli/usage_model_test.rs`)
- **Source:** [030_account_renewal_override.md AC-03](../../../docs/feature/030_account_renewal_override.md)

---

### FT-04: `clear::1` removes `_renewal_at`; `oauthAccount` preserved

- **Given:** Account `test@example.com` has both `oauthAccount` and `_renewal_at: "2026-06-29T21:00:00Z"` in `{name}.json`.
- **When:** `clp .account.renewal name::test@example.com clear::1`
- **Then:** Exits 0. `_renewal_at` key absent from file. `oauthAccount` subtree unchanged.
- **Exit:** 0
- **Source fn:** `ft04_account_renewal_clear_removes_key` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-04](../../../docs/feature/030_account_renewal_override.md)

---

### FT-05: `name::all from_now::+0m` writes current time to all accounts

- **Given:** Two accounts `alice@a.com` and `bob@a.com` in credential store; neither has `{name}.json`.
- **When:** `clp .account.renewal name::all from_now::+0m`
- **Then:** Exits 0. Both `alice@a.com.json` and `bob@a.com.json` exist and contain `_renewal_at` as an ISO-8601 string starting with `"202"`.
- **Exit:** 0
- **Note:** The cited test only checks the `"202"` year prefix on both files — it does not assert the `±10s`-of-now precision this FT case originally claimed.
- **Source fn:** `ft05_account_renewal_name_all_updates_all` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-05](../../../docs/feature/030_account_renewal_override.md)

---

### FT-06: `dry::1` prints would-be value without writing any file

- **Given:** Account `test@example.com` exists; no `{name}.json` present.
- **When:** `clp .account.renewal name::test@example.com at::2026-06-29T21:00:00Z dry::1`
- **Then:** Exits 0. Stdout contains `[dry-run]` prefix and the target timestamp. `test@example.com.json` does NOT exist after the command.
- **Exit:** 0
- **Source fn:** `ft06_account_renewal_dry_no_write` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-06](../../../docs/feature/030_account_renewal_override.md)

---

### FT-07: `at::` and `from_now::` together exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com at::2026-06-29T21:00:00Z from_now::+1h`
- **Then:** Exits 1. Stderr contains error naming both conflicting params. No file written.
- **Exit:** 1
- **Source fn:** `ft07_account_renewal_at_from_now_conflict` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-07](../../../docs/feature/030_account_renewal_override.md)

---

### FT-08: `at::` and `clear::` together exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com at::2026-06-29T21:00:00Z clear::1`
- **Then:** Exits 1. Stderr contains error naming the conflict. No file written.
- **Exit:** 1
- **Source fn:** `ft08_account_renewal_at_clear_conflict` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-08](../../../docs/feature/030_account_renewal_override.md)

---

### FT-09: `from_now::` and `clear::` together exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com from_now::+1h clear::1`
- **Then:** Exits 1. Stderr contains error naming the conflict. No file written.
- **Exit:** 1
- **Source fn:** `ft09_account_renewal_from_now_clear_conflict` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-09](../../../docs/feature/030_account_renewal_override.md)

---

### FT-10: Past `_renewal_at` auto-advanced monthly at render time

- **Given:** Account `test@example.com` has `_renewal_at: "2020-03-15T00:00:00Z"` (past timestamp, day-15).
- **When:** `clp .usage`
- **Then:** `~Renews` column for `test@example.com` shows `in Xd` format (exact, no `~` prefix) — confirms the value is treated as an exact override even after auto-advance. The stored value is NOT modified; auto-advance is read-only at render.
- **Exit:** 0
- **Note:** No live credentials required — the cited test uses `write_account()` (no token) and its own doc comment confirms the account "will be in error state but the `~Renews` column is populated from stored data." This test does not assert day-of-month preservation across auto-advance steps — it only checks the exact-vs-estimate `~` prefix. AC-10's day-of-month preservation requirement is fixed and covered separately by the BUG-329 regression tests in `tests/usage/format_tests.rs` (see `docs/algorithm/010_renewal_date_computation.md` AC-5/AC-6). See BUG-329 (`/home/alice/assistant/task/claude_profile/bug/329_auto_advance_flat_step_drifts_day_of_month.md`, closed) for the full mechanism and fix history — the underlying defect is fixed and verified in code, and the bug-tracking file's own lifecycle closure is complete.
- **Source fn:** `it151_past_renewal_at_auto_advances_in_usage` (in `tests/cli/usage_model_test.rs`)
- **Source:** [030_account_renewal_override.md AC-10](../../../docs/feature/030_account_renewal_override.md)

---

### FT-11: `~Renews` renders exact / estimated / absent correctly

- **Given (unit test):** Three `AccountQuota` structs, all evaluated at `now_secs = 1_893_456_000` (2030-01-01T00:00:00Z):
  - A: `renewal_at_opt = Some("2030-01-01T03:47:00Z")` (future) → exact
  - B: `renewal_at_opt = None`, `org_created_at_opt = Some("2025-01-15T00:00:00Z")` → estimated
  - C: `renewal_at_opt = None`, `org_created_at_opt = None` → absent
- **When:** `renews_label(renewal_at_opt, org_created_at_opt, now_secs)` for each.
- **Then:** A returns exactly `"in 3h 47m"` (no `~` prefix); B returns a `"~in "`-prefixed string containing `'d'` (day unit); C returns `"?"`.
- **Exit:** n/a (unit test)
- **Source fn:** `rl_exact_from_renewal_at`, `rl_estimate_from_org_created_at`, `rl_absent_returns_question` (in `tests/usage/format_tests.rs`)
- **Source:** [030_account_renewal_override.md AC-11](../../../docs/feature/030_account_renewal_override.md)

---

### FT-12: No operation param provided exits 1

- **Given:** Account `test@example.com` exists.
- **When:** `clp .account.renewal name::test@example.com` (no `at::`, `from_now::`, or `clear::`)
- **Then:** Exits 1. Stderr contains a usage error.
- **Exit:** 1
- **Source fn:** `ft10_account_renewal_no_operation_exits_1` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-12](../../../docs/feature/030_account_renewal_override.md)

---

### FT-13: Unknown account exits 2

- **Given:** `test@example.com` is NOT in the credential store.
- **When:** `clp .account.renewal name::test@example.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 2. Stderr contains an error message naming the account.
- **Exit:** 2
- **Source fn:** `ft11_account_renewal_unknown_account_exits_2` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-13](../../../docs/feature/030_account_renewal_override.md)

---

### FT-14: Comma-list `name::alice,bob` updates both accounts

- **Given:** Accounts `alice@a.com` and `bob@a.com` exist.
- **When:** `clp .account.renewal name::alice@a.com,bob@a.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 0. Both `alice@a.com.json` and `bob@a.com.json` contain `_renewal_at: "2026-06-29T21:00:00Z"`. Stdout contains one status line per account.
- **Exit:** 0
- **Source fn:** `ft12_account_renewal_comma_list_updates_both` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-14](../../../docs/feature/030_account_renewal_override.md)

---

### FT-15: Partial comma-list — unknown account reported; others succeed

- **Given:** `alice@a.com` exists; `unknown@a.com` does not.
- **When:** `clp .account.renewal name::alice@a.com,unknown@a.com at::2026-06-29T21:00:00Z`
- **Then:** Non-zero exit. `alice@a.com.json` contains `_renewal_at` (was processed). Stderr contains an error message for `unknown@a.com`. Stdout contains the status line for `alice@a.com`.
- **Exit:** non-zero
- **Source fn:** `ft13_account_renewal_partial_comma_list` (in `account_renewal_test.rs`)
- **Source:** [030_account_renewal_override.md AC-15](../../../docs/feature/030_account_renewal_override.md)
