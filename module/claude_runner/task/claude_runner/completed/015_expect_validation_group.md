# Expect Output Validation Group

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** module/
- **Validated By:** null
- **Validation Date:** null

## Goal

CLR has no mechanism to validate that subprocess output conforms to an expected set of values.
Automation pipelines that require a yes/no or fixed-option response must post-process output
themselves. The goal is to add the `--expect "val1|val2|..."` / `--expect-strategy` /
`--expect-retries` parameter group to provide runner-side enum output validation. Observable
end-state: `clr -p --expect "yes|no" "answer yes or no"` exits 0 when output matches and 3 when
it does not; `retry` strategy re-invokes independently up to `--expect-retries` times;
`default:<VAL>` outputs the fallback and exits 0; `w3 .test l::3` passes.

## In Scope

- `module/claude_runner/src/cli/mod.rs`:
  - Add `ExpectStrategy` enum with variants `Fail`, `Retry`, `Default(String)` (parsed from
    `"fail"`, `"retry"`, `"default:<VALUE>"`); invalid strings rejected at parse time (exit 1)
  - Add `expect: Option<String>`, `expect_strategy: Option<ExpectStrategy>`,
    `expect_retries: Option<u8>` to `CliArgs`
  - Parse `--expect <VAL>`, `--expect-strategy <STRATEGY>`, `--expect-retries <N>` in the
    hand-rolled CLI parser; reject out-of-range retries (> 255) at parse time (exit 1)
  - Apply `CLR_EXPECT`, `CLR_EXPECT_STRATEGY`, `CLR_EXPECT_RETRIES` in `apply_env_vars()`
  - Implement validation loop in `run_print_mode()`: after capturing output, if `expect` is set,
    build the expected set (split by `|`, trim, lowercase each token), compare trimmed+lowercased
    captured output against the set, dispatch per strategy:
    - Match → proceed (exit 0)
    - Mismatch + `Fail` (default) → exit 3
    - Mismatch + `Retry` → re-invoke independently up to `expect_retries` (default 0) times;
      exit 3 after all attempts exhausted
    - Mismatch + `Default(val)` → print `val` to stdout, exit 0
  - `--expect` is silently ignored when not in print mode (no message and no `--print`)
  - Add `--expect`, `--expect-strategy`, `--expect-retries` to `print_run_help()` and
    `print_ask_help()`
- New integration tests in `module/claude_runner/tests/` covering T01–T11

## Out of Scope

- `--expect` applied to `run_interactive` output (TTY passthrough — not capturable)
- JSON Schema validation via `--json-schema` (separate mechanism, unchanged)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, no `cargo fmt`),
   privacy invariant (`#[inline]` on public items), test placement (`tests/` directory only).
2. **Write failing tests** — in `module/claude_runner/tests/`, add tests for T01–T11 (see Test
   Matrix). Test categories:
   - **Parse-level (no subprocess):** T06 (`--help`), T07 (invalid strategy), T08 (out-of-range
     retries) — call the CLI parser directly or run the binary with those flags.
   - **Validation-logic (mocked subprocess output):** T01–T05, T09–T11 — use a test helper that
     feeds a fixed string as the captured output into `run_print_mode` (or stubs the subprocess),
     so no live claude invocation is required.
   Confirm all tests fail (fields absent in `CliArgs`).
3. **Add `ExpectStrategy` enum** — define the enum with a parse function; invalid strings return
   `Err` → CLI parser emits error and exits 1.
4. **Add `CliArgs` fields** — add `expect: Option<String>`, `expect_strategy: Option<ExpectStrategy>`,
   `expect_retries: Option<u8>` to `CliArgs`.
5. **Add CLI parser support** — parse `--expect`, `--expect-strategy`, `--expect-retries` in the
   hand-rolled parser. Reject invalid strategy strings at parse time (exit 1). Reject
   `--expect-retries` values > 255 at parse time (exit 1).
6. **Add env var support** — in `apply_env_vars()`, apply `CLR_EXPECT`, `CLR_EXPECT_STRATEGY`,
   `CLR_EXPECT_RETRIES` when the CLI fields are `None`.
7. **Implement validation loop in `run_print_mode()`** — after capturing output: if `expect` is
   `None`, no change. Otherwise build the expected set and compare; dispatch per strategy as
   specified in In Scope. The effective retry count when `expect_retries` is `None` is 0. Each
   retry is an independent re-invocation (new subprocess, same arguments).
8. **Update help text** — add `--expect`, `--expect-strategy`, `--expect-retries` entries to
   `print_run_help()` and `print_ask_help()`.
9. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
10. **Submit for Validation** — trigger SUBMIT transition. An independent validator executes the
    validation procedure. A NO or deviation triggers REJECT; fix all gaps, resubmit.
11. **Update task state** — on validation pass, update `task/readme.md` index, move file to
    `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | output = `"yes"`, `--expect "yes\|no"`, default strategy | match path | Exit 0 |
| T02 | output = `"maybe"`, `--expect "yes\|no"`, default strategy (`fail`) | mismatch + fail | Exit 3 |
| T03 | output = `"YES"`, `--expect "yes\|no"` | case-insensitive match | Exit 0 |
| T04 | output = `"yes\n"`, `--expect "yes\|no"` | whitespace trimmed | Exit 0 |
| T05 | `--expect "yes\|no"` in interactive mode (no message, no `--print`), dry-run | silent ignore | Assembled command does NOT contain `--expect`; exit 0 |
| T06 | `clr --help` | stdout | Contains `--expect`, `--expect-strategy`, `--expect-retries` |
| T07 | `--expect-strategy bogus_value` | parse error | Exit 1; stderr contains error |
| T08 | `--expect-retries 300` | out-of-range value | Exit 1; stderr contains error |
| T09 | output = `"maybe"`, `--expect "yes\|no"`, `--expect-strategy default:no` | default fallback | stdout = `"no"`, exit 0 |
| T10 | output = `"maybe"`, `--expect "yes\|no"`, `--expect-strategy retry`, `--expect-retries 0` | retry=0 | Exit 3 after 1 total attempt |
| T11 | output = `"maybe"` on all attempts, `--expect "yes\|no"`, `--expect-strategy retry`, `--expect-retries 2` | retry exhausted | Exit 3 after 3 total attempts (1 initial + 2 retries) |

## Acceptance Criteria

- `ExpectStrategy` enum has variants `Fail`, `Retry`, `Default(String)`
- Invalid strategy string at parse time exits 1 — verified by T07
- `--expect-retries` > 255 exits 1 at parse time — verified by T08
- Match exits 0 — verified by T01
- Mismatch with `fail` exits 3 — verified by T02
- Matching is case-insensitive and trim-based — verified by T03, T04
- `--expect` silently ignored in interactive mode (no message, no `--print`) — verified by T05
- `--help` output contains all three parameter names — verified by T06
- `default:<VAL>` outputs fallback to stdout and exits 0 — verified by T09
- Retry with 0 retries = 1 total attempt — verified by T10
- Retry exhausts N+1 total attempts then exits 3 — verified by T11
- `CLR_EXPECT`, `CLR_EXPECT_STRATEGY`, `CLR_EXPECT_RETRIES` applied in `apply_env_vars()`
- `w3 .test l::3` passes with zero failures and zero warnings

## Validation

### Checklist

**Implementation (positive)**
- [ ] `ExpectStrategy` enum with 3 variants (`Fail`, `Retry`, `Default(String)`) defined?
- [ ] Invalid strategy string rejected at parse time (exit 1) — T07?
- [ ] `--expect-retries` > 255 rejected at parse time (exit 1) — T08?
- [ ] Match path exits 0 — T01?
- [ ] Mismatch + `fail` exits 3 — T02?
- [ ] Case-insensitive trim matching — T03, T04?
- [ ] Interactive mode: `--expect` silently ignored — T05?
- [ ] Help text contains all 3 param names — T06?
- [ ] `default:<VAL>` outputs to stdout, exits 0 — T09?
- [ ] Retry total attempts = N+1 — T10, T11?
- [ ] `CLR_EXPECT`, `CLR_EXPECT_STRATEGY`, `CLR_EXPECT_RETRIES` applied in `apply_env_vars()`?
- [ ] `w3 .test l::3` passes?

**Out of Scope (absence)**
- [ ] `run_interactive` is NOT modified with `--expect` validation logic?
- [ ] `--json-schema` parsing is NOT modified?

### Measurements

- New tests: ≥ 11 passing tests (T01–T11)
- `w3 .test l::3`: 0 failures, 0 warnings
- Help text: `grep -E "expect|expect-strategy|expect-retries" <(clr --help)` → ≥ 3 matches

### Invariants

- Exit code 3 is exclusive to `--expect` mismatch — verify: `grep -n "exit(3)" module/claude_runner/src/cli/mod.rs`
  shows only the expect-validation exit paths
- `run_interactive` unmodified — verify: `grep -n "expect" module/claude_runner/src/cli/mod.rs`
  shows no references inside the `run_interactive` function body

### Anti-faking checks

- `grep -n "ExpectStrategy" module/claude_runner/src/cli/mod.rs` — must show enum definition and
  at least one match arm inside `run_print_mode`
- T05 dry-run: `clr --dry-run --expect "yes|no" "task"` — assembled describe output must NOT
  contain `--expect` (silently ignored when no-print interactive mode; note: with message present
  and default print mode active, this test should use bare interactive mode invocation)

## Related Documentation

All documentation files below are pre-existing — created as part of the doc update session that
preceded this task file. No doc creation is needed during implementation.

- `docs/cli/param/030_expect.md` — `--expect` parameter specification (pre-existing)
- `docs/cli/param/031_expect_strategy.md` — `--expect-strategy` parameter specification (pre-existing)
- `docs/cli/param/032_expect_retries.md` — `--expect-retries` parameter specification (pre-existing)
- `tests/docs/cli/param/30_expect.md` — test case index EC-1 through EC-6 (pre-existing)
- `tests/docs/cli/param/31_expect_strategy.md` — test case index (pre-existing)
- `tests/docs/cli/param/32_expect_retries.md` — test case index (pre-existing)
- `module/claude_runner/src/cli/mod.rs` — implementation target

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | Bounded to `run_print_mode` + parser + env vars + help text; Out of Scope excludes TTY and json-schema. |
| MOST Goal Quality | PASS | Concrete end-state with specific exit codes and example invocation; `w3 .test l::3` criterion. |
| Value / YAGNI | PASS | Addresses concrete pipeline gap; Out of Scope excludes speculative extensions. |
| Implementation Readiness | PASS | Step 2 categorizes parse-level vs mocked tests; docs noted as pre-existing; 11-step ordered procedure; Test Matrix has 11 rows. |

Verified: 2026-06-06. Transition: ❓ → 🎯.

## History

- **[2026-06-06]** `CREATED` — Implement `--expect` / `--expect-strategy` / `--expect-retries`
  output validation group in `run_print_mode`.
- **[2026-06-06]** `VERIFIED` — MAAV passed (4/4 dimensions, 1 fix cycle for Implementation Readiness). Transitioned ❓→🎯.
- **[2026-06-07]** `COMPLETED` — All tests pass (16/16 crates green, w3 .test l::3). Moved to completed/.
