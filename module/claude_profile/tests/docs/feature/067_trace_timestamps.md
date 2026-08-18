# Test: Feature 067 — Trace Timestamp Prefix

### Scope

- **Purpose**: Test cases for the unconditional `trace_ts()` timestamp prefix replacing the `[trace]` sentinel.
- **Source**: `docs/feature/067_trace_timestamps.md`
- **Covers**: AC-01 through AC-06

Feature behavioral requirement test cases for `docs/feature/067_trace_timestamps.md`. Tests are spread across 12 existing test files — this feature modifies diagnostic output format rather than adding new commands, so assertions in existing tests were updated rather than new test functions created. The BUG-234 MRE in `tests/usage/fetch_tests.rs` is the only dedicated structural guard.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `trace_ts()` is `pub fn` in production code, not `#[cfg(test)]`-gated | AC-01 |
| FT-02 | `trace_ts()` returns string matching `YYYY-MM-DD · HH:MM:SS UTC · ` format (BUG-338) | AC-02 |
| FT-03 | All integration trace assertions use ` · ` sentinel — no `[trace]` strings remain | AC-03 |
| FT-04 | `trace_ts()` body contains no trace-flag check — always returns timestamp | AC-04 |
| FT-05 | Touch skip trace line contains ` · touch  ` sentinel (not `[trace] touch`) | AC-03, AC-05 |
| FT-06 | BUG-234 MRE: `"{}{}  result: OK"` pattern present in production `fetch.rs` `eprintln!` | AC-06 |
| FT-07 | Fetch trace line contains ` · ` and account label without `[trace]` prefix | AC-03, AC-05 |

### Test Locations

| FT | File | Notes |
|----|------|-------|
| FT-01 | *(no test — coverage gap)* | Claim true by direct inspection of `account/mod.rs`; no structural test exists in `fetch_tests.rs` or elsewhere |
| FT-02 | `tests/usage/touch_tests.rs`, `tests/usage/api_tests_a.rs` | Regex or substring check on stderr output |
| FT-03 | All 12 test files (see Sources in feature doc) | Assertion pattern changed from `[trace]` to ` · ` |
| FT-04 | *(no test — coverage gap)* | Claim true by direct inspection of `trace_ts` in `claude_core`; no structural test exists in `fetch_tests.rs` or elsewhere |
| FT-05 | `tests/usage/touch_tests.rs:538` | Structural `src.contains(...)` check in `test_apply_touch_touch_idle_false_silent_when_trace_disabled` — not a literal `contains(" · touch  ")` stderr capture |
| FT-06 | `tests/usage/fetch_tests.rs:157` | BUG-234 MRE (`mre_bug234_result_trace_after_billing_type_override`): `src.find( r#"eprintln!( "{}{}  result: OK""# )` |
| FT-07 | `tests/cli/usage_solo_test.rs`, `tests/cli/usage_feature_test.rs` | `.filter( |l| l.contains( " · " ) )` usage — corrected file (`tests/cli/usage_test.rs` does not exist) |

### FT-01: `trace_ts()` is available in production code

- **Given:** `claude_profile_core/src/account/` is the production source tree.
- **When:** The source is inspected for `pub fn trace_ts` with no preceding `#[cfg(test)]` attribute.
- **Then:** Source match found. `trace_ts` is a `pub fn` callable from production `eprintln!` paths — not test-gated.
- **Source fn:** *(coverage gap — no structural test exists in `tests/usage/fetch_tests.rs` or elsewhere; that file's `src.find(...)` structural assertions target unrelated strings (the BUG-234 `result: OK` pattern, the Class A billing override). The claim itself is true by direct inspection: `trace_ts()` is implemented in `claude_core` and re-exported from `claude_profile_core/src/account/mod.rs`, no `#[cfg(test)]` attribute — but nothing in the suite asserts this automatically.)*

---

### FT-02: `trace_ts()` return value matches timestamp format

- **Given:** `trace_ts()` is called at any point during a `.usage trace::1` run.
- **When:** The return value is captured via stderr output or direct call.
- **Then:** Return value matches `"YYYY-MM-DD · HH:MM:SS UTC · "`: 10 date digits, space-dot-space separator, 8 time digits, space, literal `UTC` marker, trailing space-dot-space. Each trace line in stderr begins with this pattern. (Fixed per BUG-338 — `UTC` marker disambiguates from other timestamp sources sharing the same shape.)
- **Source fn:** structural/regex assertions in `tests/usage/touch_tests.rs`, `tests/usage/api_tests_a.rs`; new unit assertion `trace_ts_returns_utc_marked_timestamp` in `claude_profile_core/tests/account_test.rs` (TSK-419, see BUG-338 Refs: tests/)

---

### FT-03: No `[trace]` sentinel strings remain in test assertions

- **Given:** All 12 test files that formerly asserted `contains("[trace]")`.
- **When:** `grep -rn 'contains.*"\[trace\]"' tests/cli/ src/usage/ --include="*.rs"` is run.
- **Then:** Returns 0 matches. All assertions updated to use `contains(" · ")` or `contains(" · label  ")`.
- **Source fn:** all 12 test files updated (see Test Locations table above)

---

### FT-04: `trace_ts()` body contains no conditional guard

- **Given:** `trace_ts()` implementation in `claude_core` (re-exported from `account/mod.rs`).
- **When:** The function body is inspected structurally.
- **Then:** No `if trace`, `if enabled`, or similar conditional found in the body. The function simply formats the UTC timestamp unconditionally.
- **Source fn:** *(coverage gap — no structural test exists in `tests/usage/fetch_tests.rs` or elsewhere asserting `trace_ts()`'s own body is unconditional. The claim itself is true by direct inspection: `trace_ts()` (implemented in `claude_core`, re-exported from `claude_profile_core/src/account/mod.rs`) contains no conditional — but nothing in the suite asserts this automatically.)*

---

### FT-05: Touch skip trace line uses ` · touch  ` sentinel

- **Given:** An account is skipped during `apply_touch()` due to solo mode.
- **When:** `clp .usage touch::1 solo::1 trace::1` (non-current account present).
- **Then:** stderr contains `" · touch  "` followed by the account name and skip reason. No `"[trace] touch  "` prefix appears.
- **Source fn:** `test_apply_touch_touch_idle_false_silent_when_trace_disabled` (in `tests/usage/touch_tests.rs:538`) — corrected from an inexact `contains(" · touch  ")` paraphrase; the actual assertion is a structural source-inspection check (`src.contains("if trace { let _ = writeln!( std::io::stderr(), \"{}touch  {}  {}\", trace_ts(), aq.name, reason ); }")`) confirming the sentinel-producing `writeln!` call, not a literal captured-stderr string match. No test asserts the solo-mode-specific scenario in this Given/When directly — the closest scenario-level match, `ec8_solo_gate_skips_non_current_with_trace` (`tests/usage/touch_tests_b.rs:290`), checks `touch_skip_reason()`'s return value only, not the formatted stderr line.

---

### FT-06: BUG-234 MRE — `result: OK` eprintln! uses two-argument form

- **Given:** `src/usage/fetch.rs` production source file.
- **When:** Structural test searches for `eprintln!( "{}{}  result: OK"` in the source.
- **Then:** `src.find( r#"eprintln!( "{}{}  result: OK""# )` returns `Some(...)`. Exactly 1 match at the production site and 1 match at the structural assertion itself.
- **Source fn:** `mre_bug234_result_trace_after_billing_type_override` (in `tests/usage/fetch_tests.rs:157`) — corrected name (was cited as `mre_bug234_result_ok_uses_two_arg_eprintln`)

---

### FT-07: Fetch trace line filter captures all trace lines

- **Given:** `.usage trace::1` run with multiple accounts.
- **When:** `usage_feature_test.rs` applies `.filter( |l| l.contains( " · " ) )` to stderr lines.
- **Then:** Trace line count matches expected count. No trace lines missed or double-counted by the ` · ` filter.
- **Source fn:** filter assertion in `tests/cli/usage_feature_test.rs`

### Test Function Naming

No new `ft_NNN_` functions were added for Feature 067 — assertions were updated in existing integration tests. The BUG-234 MRE in `tests/usage/fetch_tests.rs` is named per the bug convention: `mre_bug234_result_ok_uses_two_arg_eprintln` (or equivalent; name was pre-existing).
