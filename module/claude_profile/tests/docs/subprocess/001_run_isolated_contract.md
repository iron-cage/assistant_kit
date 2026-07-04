# Subprocess 001: run_isolated Contract

AC test cases for `docs/subprocess/001_run_isolated_contract.md`. Tests the
`run_isolated()` API contract — sole-caller invariant, argument contract
(`["--print", "."]`), and `expiresAt=1` manipulation ordering.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Sole authorized caller — `run_isolated()` not called directly from `src/` | Invariant | ✅ |
| AC-2 | `run_isolated()` source doc contains sole-caller warning | Invariant | ✅ |
| AC-3 | Only valid args `["--print", "."]` used in refresh path | Invariant | ✅ |
| AC-4 | `expiresAt=1` manipulation precedes every `run_isolated()` call | Invariant | ✅ |

---

### AC-1: Sole authorized caller — `run_isolated()` not called directly from `src/`

- **Given:** All Rust source files under `src/` in the `claude_profile` crate.
- **When:** Grep for direct `run_isolated(` calls is performed.
- **Then:** Zero occurrences found. All credential refresh operations MUST go through
  `account::refresh_account_token()`. Direct `run_isolated()` calls from `src/` are
  forbidden per `invariant/008_single_token_refresh_entry.md`.
- **Source fn:** `single_token_refresh_entry_in1_src_contains_zero_run_isolated_calls` in
  `tests/cli/invariant_test.rs`
- **Source:** [subprocess/001_run_isolated_contract.md](../../../docs/subprocess/001_run_isolated_contract.md)

---

### AC-2: `run_isolated()` source has sole-caller warning comment

- **Given:** The source file defining `pub fn run_isolated` in `claude_runner_core`.
- **When:** The function body or doc comment is inspected.
- **Then:** A warning or guard comment is present indicating `refresh_account_token()` is the
  only authorized caller. This structural guard prevents future callers from bypassing the
  single-entry-point invariant when writing new code.
- **Source fn:** `single_token_refresh_entry_in2_run_isolated_doc_has_warning` in
  `tests/cli/invariant_test.rs`
- **Source:** [subprocess/001_run_isolated_contract.md](../../../docs/subprocess/001_run_isolated_contract.md)

---

### AC-3: Only valid args `["--print", "."]` used in refresh path

- **Given:** The `apply_refresh()` code path that invokes `refresh_account_token()`.
- **When:** The trace shows the `run_isolated` invocation arguments.
- **Then:** The invocation uses `["--print", "."]` — the ONLY correct argument vector for
  credential refresh pings. Fix BUG-169: empty args `[]` causes Claude to detect no task and
  exit without OAuth refresh, producing `credentials=None` always. The `["--print", "."]`
  contract is confirmed to be present in the live invocation path.
- **Source fn:** `test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [subprocess/001_run_isolated_contract.md](../../../docs/subprocess/001_run_isolated_contract.md)

---

### AC-4: `expiresAt=1` manipulation precedes every `run_isolated()` call

- **Given:** The source code path of `refresh_account_token()`.
- **When:** Code inspection confirms the sequence of operations before `run_isolated()`.
- **Then:** `expiresAt` is set to `"1"` in the in-memory credential copy BEFORE the
  `run_isolated()` call in every code path that leads to `run_isolated()`. This forces Claude
  CLI to classify the AT as expired, triggering the RT→AT+RT OAuth exchange. Without this,
  a valid AT causes the subprocess to use it as-is, returning `credentials=None` silently
  and leaving the RT to age toward server-side expiry.
- **Source fn:** `single_token_refresh_entry_in3_expires_manipulation_before_run_isolated` in
  `tests/cli/invariant_test.rs`
- **Source:** [subprocess/001_run_isolated_contract.md](../../../docs/subprocess/001_run_isolated_contract.md)
