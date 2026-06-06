# BUG-247: Forward Stdout to Stderr on Failure in run_print_mode

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

`run_print_mode()` discards stdout content when `exit_code != 0`. When `claude` writes a diagnostic (e.g. `"API Error: 529 overloaded"`) to stdout and exits non-zero, the text is silently lost — only the classified label (`"Error: API error (exit 1)"`) reaches the caller. The goal is to add a stdout-to-stderr forward for the failure path so operators always receive the full diagnostic payload. Observable end-state: a test supplying a mock subprocess that writes text to stdout and exits 1 confirms that text appears on clr's stderr; `w3 .test l::3` passes.

## In Scope

- `module/claude_runner/src/cli/mod.rs` — `run_print_mode()`: after the stderr forward (`eprint!("{}", output.stderr)`), add a stdout-to-stderr forward when `output.exit_code != 0 && !output.stdout.is_empty()`; insert before the `classify_error()` block
- `module/claude_runner/tests/` — new test file `bug_reproducers_247_test.rs` with MRE tests reproducing the gap (mock subprocess that exits 1 with stdout content; verify that content appears on clr's stderr)
- Source comment on the new code block in the Fix(BUG-247) format (3 fields: Root cause, Pitfall)

## Out of Scope

- Changes to `classify_error()` or `ExecutionOutput` in `claude_runner_core`
- Changes to `run_interactive()` — interactive mode does not capture stdout
- Any verbosity gating on the stdout forward — mirrors the unconditional stderr forward above it
- Changes to dry-run, trace, or ask/isolated/refresh paths

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Forward must be unconditional (no verbosity gate) — mirrors the existing stderr forward pattern
- The `Fix(BUG-247)` source comment must include: `Root cause:` (why forward was missing) and `Pitfall:` (why no verbosity gate)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, no `cargo fmt`), test placement (`tests/` directory only), Fix comment format (3 fields).
2. **Write MRE test (failing)** — in `module/claude_runner/tests/bug_reproducers_247_test.rs`: create a test `stdout_forwarded_to_stderr_on_failure` that runs a mock subprocess (via a real integration helper or path trick with a shell script in `TMPDIR`) that prints to stdout and exits 1, then asserts the text appears on clr's stderr. Confirm it fails under current code.
3. **Apply fix** — in `run_print_mode()`: after `if !output.stderr.is_empty() { eprint!(...) }`, add:
   ```rust
   // Fix(BUG-247): forward stdout to stderr when exit_code != 0 so diagnostic
   //   text written by claude to stdout reaches the caller.
   // Root cause: the success-path stdout forward (`print!("{out}")`) never executes
   //   on failure; stdout content is silently discarded after the non-zero exit branch.
   // Pitfall: no verbosity gate — mirrors the unconditional stderr forward above;
   //   raw subprocess output is always forwarded regardless of verbosity level.
   if output.exit_code != 0 && !output.stdout.is_empty()
   {
     eprint!( "{}", output.stdout );
   }
   ```
4. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
5. **Submit for Validation** — trigger SUBMIT transition. Independent validator executes the validation procedure. A NO triggers REJECT; fix all gaps, resubmit.
6. **Update task state** — on validation pass, set ✅ in `task/readme.md`, recalculate advisability to 0, re-sort, move to `task/completed/`; update `bug/readme.md` to `fixed`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Subprocess exits 1 with content on stdout | `run_print_mode` failure path | stdout content appears on clr's stderr |
| T02 | Subprocess exits 1 with content on both stdout and stderr | `run_print_mode` failure path | both stdout and stderr content appear on clr's stderr |
| T03 | Subprocess exits 1 with empty stdout | `run_print_mode` failure path | no spurious blank line on stderr from empty stdout |
| T04 | Subprocess exits 0 with content on stdout | `run_print_mode` success path | stdout content appears on clr's stdout (unchanged) |
| T05 | Subprocess exits 2 (rate-limit) with content on stdout | `run_print_mode` failure path | stdout content forwarded to stderr; exit code 2 propagated |

## Acceptance Criteria

- `run_print_mode()` forwards `output.stdout` to stderr when `exit_code != 0 && !output.stdout.is_empty()`
- No verbosity gate on the stdout forward — the forward is unconditional (mirrors stderr forward)
- T01–T05 all have corresponding passing tests in `bug_reproducers_247_test.rs`
- Existing tests unaffected (`w3 .test l::3` passes at zero failures/warnings)
- `Fix(BUG-247)` source comment present with `Root cause:` and `Pitfall:` fields

## Validation

### Checklist

**Implementation (positive)**
- [ ] `run_print_mode()` contains `eprint!("{}", output.stdout)` guarded by `exit_code != 0 && !output.stdout.is_empty()`?
- [ ] The guard appears BEFORE the `classify_error()` block (same position relative to stderr forward)?
- [ ] No verbosity gate on the new forward?
- [ ] `Fix(BUG-247)` comment present with all 3 fields?
- [ ] T01–T05 all have passing tests?
- [ ] `w3 .test l::3` passes at zero failures/warnings?

**Out of Scope (absence)**
- [ ] `run_interactive()` is unmodified?
- [ ] `classify_error()` in `claude_runner_core` is unmodified?
- [ ] `ExecutionOutput` struct is unmodified?

### Measurements

- Test count: ≥ 5 new passing tests (T01–T05)
- `w3 .test l::3`: 0 failures, 0 warnings

### Invariants

- Forward is unconditional: `grep -A5 "BUG-247" module/claude_runner/src/cli/mod.rs` must NOT contain `shows_` or `verbosity` in the guarding condition

### Anti-faking checks

- `grep -n "output.stdout" module/claude_runner/src/cli/mod.rs` — must show at least two references: one in the failure path (eprint) and one in the success path (print)

## Related Documentation

- `module/claude_runner_core/docs/failure_mode/002_diagnostic_on_stdout.md` — documents the root failure mode (clr Response section describes this gap)
- `module/claude_runner_core/docs/failure_mode/readme.md` — Silent Fails Table row 002 with clr Response column
- `module/claude_runner/task/claude_runner/bug/247_stdout_swallowed_on_failure.md` — formal bug report (BUG-247)

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | In Scope: 3 concrete bounded items; Out of Scope: 4 meaningful exclusions; observable outcome: T01–T05 pass + `w3 .test l::3` green. |
| MOST Goal Quality | PASS | Motivated (diagnostic lost on failure); Observable (text appears on clr stderr, w3 passes); Scoped (one guard block + one test file); Testable (mechanical grep + test runner). |
| Value / YAGNI | PASS | No null-hypothesis defense survives; concrete committed need (BUG-247 filed, failure_mode/002 documented); no scope creep. |
| Implementation Readiness | PASS | Steps numbered 1–6, ordered, unambiguous; Test Matrix 5 concrete rows; file paths and function names specific. |

Verified: 2026-06-07. Transition: ❓ → 🎯.

## History

- **[2026-06-07]** `CREATED` — Forward stdout to stderr when exit_code != 0 in `run_print_mode()` to fix BUG-247.
- **[2026-06-07]** `VERIFIED` — MAAV passed (4/4 dimensions). Transitioned ❓→🎯.
- **[2026-06-07]** `COMPLETED` — All tests pass (16/16 crates green, w3 .test l::3). Moved to completed/.
