# Test: Feature 067 — Trace Timestamp Prefix

Feature behavioral requirement test cases for `docs/feature/067_trace_timestamps.md`. Tests are spread across 12 existing test files — this feature modifies diagnostic output format rather than adding new commands, so assertions in existing tests were updated rather than new test functions created. The BUG-234 MRE in `tests/usage/fetch_tests.rs` is the only dedicated structural guard.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `trace_ts()` is `pub fn` in production code, not `#[cfg(test)]`-gated | AC-01 |
| FT-02 | `trace_ts()` returns string matching `YYYY-MM-DD · HH:MM:SS · ` format | AC-02 |
| FT-03 | All integration trace assertions use ` · ` sentinel — no `[trace]` strings remain | AC-03 |
| FT-04 | `trace_ts()` body contains no trace-flag check — always returns timestamp | AC-04 |
| FT-05 | Touch skip trace line contains ` · touch  ` sentinel (not `[trace] touch`) | AC-03, AC-05 |
| FT-06 | BUG-234 MRE: `"{}{}  result: OK"` pattern present in production `fetch.rs` `eprintln!` | AC-06 |
| FT-07 | Fetch trace line contains ` · ` and account label without `[trace]` prefix | AC-03, AC-05 |

### Test Locations

| FT | File | Notes |
|----|------|-------|
| FT-01 | `tests/usage/fetch_tests.rs` | Structural test via `src.find( ... )` on `account.rs` source |
| FT-02 | `tests/usage/touch_tests.rs`, `tests/usage/api_tests_a.rs` | Regex or substring check on stderr output |
| FT-03 | All 12 test files (see Sources in feature doc) | Assertion pattern changed from `[trace]` to ` · ` |
| FT-04 | `tests/usage/fetch_tests.rs` | Structural test: `trace_ts` fn body does not contain `if trace` |
| FT-05 | `tests/usage/touch_tests.rs` | `contains( " · touch  " )` assertions |
| FT-06 | `tests/usage/fetch_tests.rs` | BUG-234 MRE: `src.find( r#"eprintln!( "{}{}  result: OK""# )` |
| FT-07 | `tests/cli/usage_test.rs`, `tests/cli/usage_feature_test.rs` | `.filter( |l| l.contains( " · " ) )` usage |

### FT-01: `trace_ts()` is available in production code

- **Given:** `claude_profile_core/src/account.rs` is the production source file.
- **When:** The source is inspected for `pub fn trace_ts` with no preceding `#[cfg(test)]` attribute.
- **Then:** Source match found. `trace_ts` is a `pub fn` callable from production `eprintln!` paths — not test-gated.
- **Source fn:** structural test in `tests/usage/fetch_tests.rs`

---

### FT-02: `trace_ts()` return value matches timestamp format

- **Given:** `trace_ts()` is called at any point during a `.usage trace::1` run.
- **When:** The return value is captured via stderr output or direct call.
- **Then:** Return value matches `"YYYY-MM-DD · HH:MM:SS · "`: 10 date digits, space-dot-space separator, 8 time digits, trailing space-dot-space. Each trace line in stderr begins with this pattern.
- **Source fn:** structural/regex assertions in `tests/usage/touch_tests.rs`, `tests/usage/api_tests_a.rs`

---

### FT-03: No `[trace]` sentinel strings remain in test assertions

- **Given:** All 12 test files that formerly asserted `contains("[trace]")`.
- **When:** `grep -rn 'contains.*"\[trace\]"' tests/cli/ src/usage/ --include="*.rs"` is run.
- **Then:** Returns 0 matches. All assertions updated to use `contains(" · ")` or `contains(" · label  ")`.
- **Source fn:** all 12 test files updated (see Test Locations table above)

---

### FT-04: `trace_ts()` body contains no conditional guard

- **Given:** `trace_ts()` implementation in `account.rs`.
- **When:** The function body is inspected structurally.
- **Then:** No `if trace`, `if enabled`, or similar conditional found in the body. The function simply formats the UTC timestamp unconditionally.
- **Source fn:** structural test in `tests/usage/fetch_tests.rs`

---

### FT-05: Touch skip trace line uses ` · touch  ` sentinel

- **Given:** An account is skipped during `apply_touch()` due to solo mode.
- **When:** `clp .usage touch::1 solo::1 trace::1` (non-current account present).
- **Then:** stderr contains `" · touch  "` followed by the account name and skip reason. No `"[trace] touch  "` prefix appears.
- **Source fn:** `contains( " · touch  " )` assertions in `tests/usage/touch_tests.rs`

---

### FT-06: BUG-234 MRE — `result: OK` eprintln! uses two-argument form

- **Given:** `src/usage/fetch.rs` production source file.
- **When:** Structural test searches for `eprintln!( "{}{}  result: OK"` in the source.
- **Then:** `src.find( r#"eprintln!( "{}{}  result: OK""# )` returns `Some(...)`. Exactly 1 match at the production site and 1 match at the structural assertion itself.
- **Source fn:** `mre_bug234_result_ok_uses_two_arg_eprintln` (in `tests/usage/fetch_tests.rs`)

---

### FT-07: Fetch trace line filter captures all trace lines

- **Given:** `.usage trace::1` run with multiple accounts.
- **When:** `usage_feature_test.rs` applies `.filter( |l| l.contains( " · " ) )` to stderr lines.
- **Then:** Trace line count matches expected count. No trace lines missed or double-counted by the ` · ` filter.
- **Source fn:** filter assertion in `tests/cli/usage_feature_test.rs`

### Test Function Naming

No new `ft_NNN_` functions were added for Feature 067 — assertions were updated in existing integration tests. The BUG-234 MRE in `tests/usage/fetch_tests.rs` is named per the bug convention: `mre_bug234_result_ok_uses_two_arg_eprintln` (or equivalent; name was pre-existing).
