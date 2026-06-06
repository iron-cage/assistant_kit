# Error Classification in CLR

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** module/
- **Validated By:** null
- **Validation Date:** null

## Goal

CLR currently emits a generic `"Error: Claude exited without output (possible rate limit or quota exhaustion)"` message for every non-zero exit with empty stderr — rate limits, auth failures, and API errors all produce identical output. The bash automation layer compensates by re-scanning `wish .jobs` daemon output with its own pattern table, adding latency and duplicating detection logic that belongs in CLR. The goal is to add an `ErrorKind` enum and `classify_error()` method to `ExecutionOutput` in `claude_runner_core`, and update `run_print_mode` in `claude_runner` to emit a labeled diagnostic (`"Error: rate limit (exit 2)"`, `"Error: auth error (exit 1)"`, `"Error: API error (exit 1)"`) instead of the generic message. Observable end-state: `w3 .test l::3` passes with new tests covering all `ErrorKind` variants; CLR stderr for rate-limit exits contains `"Error: rate limit"` not `"possible rate limit or quota exhaustion"`.

## In Scope

- `module/claude_runner_core/src/types.rs`: add `ErrorKind` enum (`RateLimit`, `ApiError`, `AuthError`, `Signal`, `Unknown`), `ERROR_PATTERNS` const array, `classify_error(&self) -> Option<ErrorKind>` method on `ExecutionOutput`
- `module/claude_runner/src/cli/mod.rs`: replace the BUG-037 generic message block with a `classify_error()` match that emits a labeled diagnostic per variant
- New unit tests for `classify_error()` in `module/claude_runner_core/tests/`
- New integration tests verifying CLR stderr output format for each `ErrorKind` variant in `module/claude_runner/tests/`

## Out of Scope

- `ErrorKind::ContextLimit` and `ErrorKind::Timeout` variants — no silent-failure detection path exists for these; context limit and timeout both produce visible output before exit, so they are not caught by the empty-stderr + non-zero-exit detection path
- Updating `_block_error_label()` in the bash automation layer — that is a separate codebase; updating it to consume CLR's new labeled output is a future task
- Changing exit code behavior — BUG-239 exit code propagation is unchanged
- `run_interactive` mode — stderr is not captured in TTY passthrough; classification requires captured output

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on inline attributes (privacy invariant: all public items require `#[inline]`), code style (2-space indent, no `cargo fmt`), and test placement (`tests/` directory only).
2. **Write Test Matrix** — populate every row before opening any test file. The matrix is the contract; tests implement it.
3. **Write failing tests** — in `claude_runner_core/tests/`, add unit tests for `classify_error()` covering all `ErrorKind` variants. In `claude_runner/tests/`, add integration tests verifying CLR stderr output format. Confirm tests fail (types absent / wrong output).
4. **Implement `ErrorKind` and `classify_error()`** — in `claude_runner_core/src/types.rs`, create new code from scratch: (a) a `pub enum ErrorKind` with variants `RateLimit`, `ApiError`, `AuthError`, `Signal`, `Unknown`; (b) a `const ERROR_PATTERNS: &[(&str, ErrorKind)]` mapping each error string to its variant (patterns: `"You've hit your limit"` → `RateLimit`; `"Your organization does not have access to Claude"` → `AuthError`; `"API Error: "` → `ApiError`); (c) `impl ExecutionOutput { #[inline] pub fn classify_error(&self) -> Option<ErrorKind> { ... } }` scanning stderr+stdout against the pattern table, then falling back to exit code semantics. All public items must have `#[inline]` and `///` doc comments.
5. **Implement labeled diagnostic in `run_print_mode`** — in `claude_runner/src/cli/mod.rs`, replace the BUG-037 generic message block with a match on `output.classify_error()`. Each arm emits `"Error: {label} (exit {code})"`. The `None` arm is silent (success).
6. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
7. **Refactor if needed** — ensure no function exceeds 50 lines, no duplication, all public items have `///` doc comments. Tests must still pass after.
8. **Submit for Validation** — trigger SUBMIT transition. An independent validator executes the validation procedure. A NO or deviation triggers REJECT; fix all gaps, resubmit.
9. **Update task state** — on validation pass, set ✅ in `task/readme.md`, recalculate advisability to 0 (Priority=0), re-sort index, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `exit_code=2`, `stderr=""`, `stdout=""` | `classify_error()` | `Some(ErrorKind::RateLimit)` |
| T02 | `exit_code=0`, `stderr=""`, `stdout=""` | `classify_error()` | `None` |
| T03 | `exit_code=1`, `stderr="You've hit your limit"` | `classify_error()` | `Some(ErrorKind::RateLimit)` |
| T04 | `exit_code=1`, `stdout="Your organization does not have access to Claude"` | `classify_error()` | `Some(ErrorKind::AuthError)` |
| T05 | `exit_code=1`, `stderr="API Error: 529 ..."` | `classify_error()` | `Some(ErrorKind::ApiError)` |
| T06 | `exit_code=130`, `stderr=""`, `stdout=""` | `classify_error()` | `Some(ErrorKind::Signal)` |
| T07 | `exit_code=143`, `stderr=""`, `stdout=""` | `classify_error()` | `Some(ErrorKind::Signal)` |
| T08 | `exit_code=1`, `stderr=""`, `stdout=""` | `classify_error()` | `Some(ErrorKind::Unknown)` |
| T09 | mock `ExecutionOutput` with `exit_code=2`, empty output → `run_print_mode` | CLR stderr | contains `"Error: rate limit (exit 2)"` |
| T10 | mock output with auth pattern in stdout → `run_print_mode` | CLR stderr | contains `"Error: auth error"` |
| T11 | pattern `"API Error: "` appears in `stderr` with `exit_code=1` | `classify_error()` | `Some(ErrorKind::ApiError)`, NOT `Unknown` |
| T12 | pattern `"Your organization does not have access to Claude"` in `stderr` (not `stdout`) | `classify_error()` | `Some(ErrorKind::AuthError)` |

## Acceptance Criteria

-   `ErrorKind` enum is `pub` with variants: `RateLimit`, `ApiError`, `AuthError`, `Signal`, `Unknown`
-   `classify_error()` returns `None` when `exit_code == 0` — verified by T02
-   `classify_error()` returns `Some(RateLimit)` for `exit_code == 2` with no pattern match — verified by T01
-   `classify_error()` returns `Some(RateLimit)` when output contains `"You've hit your limit"` — verified by T03
-   `classify_error()` returns `Some(AuthError)` when output contains `"Your organization does not have access to Claude"` — verified by T04 and T12
-   `classify_error()` returns `Some(ApiError)` when output contains `"API Error: "` and no higher-priority pattern fires — verified by T05 and T11
-   `classify_error()` returns `Some(Signal)` for exit codes 130 and 143 — verified by T06 and T07
-   `classify_error()` returns `Some(Unknown)` for non-zero exits with no pattern match and no signal code — verified by T08
-   CLR stderr contains `"Error: rate limit (exit 2)"` for rate-limit condition — verified by T09
-   CLR stderr contains `"Error: auth error"` for auth error condition — verified by T10
-   The string `"possible rate limit or quota exhaustion"` does NOT appear in CLR stderr output for any condition — negative criterion
-   All public items (`ErrorKind`, `classify_error`) have `#[inline]` attribute (privacy invariant)
-   `w3 .test l::3` passes with zero failures, zero warnings after implementation

## Validation

### Checklist

**Implementation (positive)**
-   [ ] All Test Matrix rows (T01–T12) have a corresponding passing test?
-   [ ] `classify_error()` returns `None` for `exit_code == 0` regardless of stderr/stdout content?
-   [ ] Pattern priority is correct: auth pattern takes precedence over `"API Error: "` for 401 responses?
-   [ ] `#[inline]` present on `ErrorKind` methods and `classify_error()`?
-   [ ] No function in the implementation exceeds 50 lines?
-   [ ] Generic message `"possible rate limit or quota exhaustion"` is ABSENT from output — confirmed by grep on built binary?
-   [ ] `w3 .test l::3` passes at zero failures/warnings?

**Out of Scope (absence)**
-   [ ] `ErrorKind` enum has exactly 5 variants — `ContextLimit` and `Timeout` are NOT present?
-   [ ] Exit code propagation in `run_print_mode` is unchanged — `std::process::exit(output.exit_code)` line is preserved (BUG-239 fix intact)?
-   [ ] `run_interactive` code path is NOT modified — only `run_print_mode` touches the new classification logic?
-   [ ] No files outside `module/claude_runner_core/` and `module/claude_runner/` are modified?

### Measurements

-   Test count increase: ≥ 12 new passing tests (T01–T12)
-   `w3 .test l::3` result: 0 failures, 0 warnings
-   `grep -r "possible rate limit" module/claude_runner/` count: 0 (removed)
-   Call site count for `classify_error()` in `run_print_mode`: ≥ 1

### Invariants

-   Privacy invariant holds: all newly added public items have `#[inline]` — verify: `grep -n "#\[inline\]" module/claude_runner_core/src/types.rs` shows inline attributes preceding `classify_error` and `ErrorKind` methods
-   Testing invariant holds: pass count ≥ baseline + 12 after changes, skip count unchanged — verify: `cargo nextest run --all-features 2>&1 | tail -5`
-   `classify_error()` never panics — it returns `Option<ErrorKind>`, not `Result` or unreachable branches — verify: `grep -n "unreachable\|panic!" module/claude_runner_core/src/types.rs` returns 0 matches in `classify_error`

### Anti-faking checks

-   Run `grep -r "possible rate limit" module/` — must return 0 results; if 1+, the old message was not replaced
-   Run `cargo test classify_error` — must show ≥ 12 passing tests; if fewer, Test Matrix rows are unimplemented
-   Run `clr --dry-run ""` with a mocked exit-2 subprocess and verify stderr contains `"rate limit"`, not the generic phrase

## Related Documentation

- `docs/error/001_rate_limit_reached.md` — CLR Detection section documents the `RateLimit` variant
- `docs/error/002_authentication_failed.md` — CLR Detection section documents the `AuthError` variant
- `docs/error/005_api_overloaded.md` — CLR Detection section documents the `ApiError` variant
- `docs/integration/001_consumer_integration.md` — Error classification integration point documents `classify_error()` usage
- `module/claude_runner_core/src/types.rs` — implementation target for `ErrorKind` and `classify_error()`
- `module/claude_runner/src/cli/mod.rs` — implementation target for labeled diagnostic (replaces BUG-037 block)

## Verification Findings

First VERIFY run — 2026-06-06:

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | — |
| MOST Goal Quality | PASS | — |
| Value / YAGNI | PASS | — |
| Implementation Readiness | FAIL | Verifier flagged "ErrorKind enum missing from target file" and "classify_error() absent" — these reflect the pre-implementation state, not a task file gap. The real actionable finding: Step 4 was under-specified for the const pattern table format. Fixed: Step 4 now explicitly enumerates the enum variants, const structure, and method signature. |

Re-verification of Implementation Readiness triggered after fix.

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | Scope tightly bounded: two files + new tests; no creep. |
| MOST Goal Quality | PASS | Observable end-state is concrete and measurable (`w3 .test l::3` passes, specific stderr strings, negative criterion). |
| Value / YAGNI | PASS | Replaces real production pain (bash layer duplicates detection logic, adds latency). No speculative variants added. |
| Implementation Readiness | PASS | Step 4 explicitly enumerates enum variants, const format with three concrete pattern examples, and method signature. All target files identified. |

Verified: 2026-06-06. Transition: ❓ → 🎯.

## History

- **[2026-06-06]** `CREATED` — Add `ErrorKind` enum and `classify_error()` to replace CLR's generic silent-failure message with labeled per-type diagnostics.
- **[2026-06-06]** `VERIFIED` — MAAV passed (4/4 dimensions). Transitioned ❓→🎯.
- **[2026-06-06]** `IMPLEMENTED` — `ErrorKind` enum + `ERROR_PATTERNS` + `classify_error()` added to `types.rs`; BUG-037 block replaced in `cli/mod.rs`; 12 unit tests in `classify_error_test.rs` + 2 integration tests in `error_classification_test.rs`; 16/16 crates green via `./verb/test`.
- **[2026-06-06]** `COMPLETED` — All acceptance criteria met; test count +14 (427 core, 445 runner); generic phrase absent from codebase; `ErrorKind` exported from public API.
- **[2026-06-06]** `VERIFIED` — All 4 MAAV dimensions PASS after one fix cycle (Step 4 under-specification corrected).
