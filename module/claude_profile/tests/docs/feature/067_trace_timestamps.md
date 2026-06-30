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

### FT Case Descriptions

**FT-01** — `trace_ts()` production availability
Structural check: `claude_profile_core/src/account.rs` contains `pub fn trace_ts` without a preceding `#[cfg(test)]` attribute. The function is callable from production eprintln! paths.
Expected: source match found; no `cfg(test)` on the `trace_ts` fn definition.

**FT-02** — Timestamp format
When `trace_ts()` is called, the return value matches the pattern `"YYYY-MM-DD · HH:MM:SS · "`: 10 date digits, space-dot-space separator, 8 time digits, trailing space-dot-space.
Expected: each trace line in stderr output begins with a string matching this pattern; test assertions confirm ` · ` presence in trace output.

**FT-03** — No `[trace]` sentinel in test assertions
All assertions across 12 test files that formerly used `contains("[trace]")` now use `contains(" · ")` or `contains(" · label  ")`. Zero `[trace]` assertion strings remain in the test suite.
Expected: `grep -rn 'contains.*"\[trace\]"' tests/cli/ src/usage/ --include="*.rs"` returns 0 matches.

**FT-04** — `trace_ts()` is unconditional
The body of `trace_ts()` in `account.rs` does not contain an `if trace` or `if enabled` guard. It simply formats the UTC timestamp.
Expected: structural source check finds no conditional in `trace_ts()` body; every call site wraps the eprintln! in its own `if trace { ... }` guard.

**FT-05** — Touch skip trace format
When an account is skipped during `apply_touch()` due to solo mode, the stderr line contains `" · touch  "` followed by the account name and skip reason. Replaces former `"[trace] touch  "` prefix.
Expected: `touch_tests.rs` assertions using `contains( " · touch  " )` pass.

**FT-06** — BUG-234 MRE structural guard
The production `eprintln!` in `src/usage/fetch.rs` that emits the `result: OK` line uses the two-argument form `"{}{}  result: OK"` (timestamp + label as separate arguments). The BUG-234 MRE structural test asserts this exact string is present in the source file.
Expected: `src.find( r#"eprintln!( "{}{}  result: OK""# )` returns `Some(...)` — exactly 1 match at the production eprintln! site and 1 match at the structural assertion itself.

**FT-07** — Fetch trace line format
Trace output during `.usage trace::1` passes through lines containing `" · "` where account name and fetch action are emitted. The `usage_feature_test.rs` filter `.filter( |l| l.contains( " · " ) )` correctly captures all trace lines.
Expected: trace line count matches expected count in usage_feature_test.rs; no lines are missed or double-counted by the ` · ` filter.

### Test Function Naming

No new `ft_NNN_` functions were added for Feature 067 — assertions were updated in existing integration tests. The BUG-234 MRE in `tests/usage/fetch_tests.rs` is named per the bug convention: `mre_bug234_result_ok_uses_two_arg_eprintln` (or equivalent; name was pre-existing).
