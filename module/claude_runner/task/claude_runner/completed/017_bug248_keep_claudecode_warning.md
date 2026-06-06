# BUG-248: Warn When --keep-claudecode Disables CLAUDECODE Protection

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

When `--keep-claudecode` is passed and `CLAUDECODE` is set in the environment, `clr` silently allows the child process to inherit `CLAUDECODE=1`. No warning is emitted. The goal is to add a warning at verbosity ≥ 2 (shows_warnings) when both conditions are true: protection explicitly disabled via `--keep-claudecode` AND `CLAUDECODE` is present in the parent environment. Observable end-state: `clr --keep-claudecode --dry-run "test"` with `CLAUDECODE=1` set prints a warning line to stderr; `w3 .test l::3` passes.

## In Scope

- `module/claude_runner/src/cli/mod.rs` — `run_built_command()`: after computing `verbosity`, add a warning block checking `cli.keep_claudecode && verbosity.shows_warnings() && std::env::var("CLAUDECODE").is_ok()`
- `module/claude_runner/tests/` — new test file `bug_reproducers_248_test.rs` with MRE tests: (a) with CLAUDECODE set + `--keep-claudecode`, warning appears on stderr; (b) with CLAUDECODE set + no `--keep-claudecode`, no warning; (c) with `--keep-claudecode` + no CLAUDECODE in env, no warning; (d) with `--verbosity 1` (below shows_warnings threshold), no warning even with both conditions
- Source comment on the new code block in the Fix(BUG-248) format (3 fields: Root cause, Pitfall)

## Out of Scope

- Changes to `build_claude_command()` or the `apply_env_vars()` path
- Changes to `classify_error()` or `ExecutionOutput`
- Making the warning fatal (non-zero exit) — it is informational only
- Warning when `CLAUDECODE` is not in the environment (no false positives)
- Changes to dry-run, isolated, or refresh paths (only `run_built_command()`)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Warning is gated on `verbosity.shows_warnings()` (level ≥ 2) — not emitted at levels 0 or 1
- Warning text must include both `--keep-claudecode` and `CLAUDECODE` in the message so the cause is clear
- The `Fix(BUG-248)` source comment must include: `Root cause:` (why no warning existed) and `Pitfall:` (why verbosity gate is correct here)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, no `cargo fmt`), test placement (`tests/` directory only), Fix comment format (3 fields).
2. **Write MRE tests (failing)** — in `module/claude_runner/tests/bug_reproducers_248_test.rs`:
   - `T01`: set `CLAUDECODE=1` in test env + `--keep-claudecode --dry-run` → assert stderr contains warning
   - `T02`: no `CLAUDECODE` + `--keep-claudecode --dry-run` → assert no warning
   - `T03`: `CLAUDECODE=1` + no `--keep-claudecode --dry-run` → assert no warning
   - `T04`: `CLAUDECODE=1` + `--keep-claudecode --verbosity 1 --dry-run` → assert no warning (below threshold)
   - Confirm T01 fails under current code.
3. **Apply fix** — in `run_built_command()` after `let verbosity = cli.verbosity.unwrap_or_default();`, add:
   ```rust
   // Fix(BUG-248): warn when --keep-claudecode disables CLAUDECODE protection
   //   and CLAUDECODE is actually set in the parent environment.
   // Root cause: no warning was implemented when protection is disabled;
   //   the consequence (child in nested-agent mode) is non-obvious.
   // Pitfall: gate on shows_warnings() (level >= 2) so operators who set
   //   --verbosity 0/1 for silence still get silence; the warning is
   //   informational, not fatal — the user's intent is respected.
   if cli.keep_claudecode
     && verbosity.shows_warnings()
     && std::env::var( "CLAUDECODE" ).is_ok()
   {
     eprintln!(
       "Warning: --keep-claudecode is set and CLAUDECODE is present in \
        environment; child claude will run in nested-agent mode"
     );
   }
   ```
4. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
5. **Submit for Validation** — trigger SUBMIT transition. Independent validator executes the validation procedure. A NO triggers REJECT; fix all gaps, resubmit.
6. **Update task state** — on validation pass, set ✅ in `task/readme.md`, recalculate advisability to 0, re-sort, move to `task/completed/`; update `bug/readme.md` to `fixed`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `CLAUDECODE=1` in env + `--keep-claudecode` | default verbosity (3) | warning line on stderr containing `--keep-claudecode` and `CLAUDECODE` |
| T02 | `CLAUDECODE` absent + `--keep-claudecode` | default verbosity | no warning |
| T03 | `CLAUDECODE=1` in env, no `--keep-claudecode` | default verbosity | no warning |
| T04 | `CLAUDECODE=1` + `--keep-claudecode` + `--verbosity 1` | verbosity 1 (errors only) | no warning (below shows_warnings threshold) |
| T05 | `CLAUDECODE=1` + `--keep-claudecode` + `--verbosity 2` | verbosity 2 (shows_warnings) | warning present |
| T06 | `CLAUDECODE=1` + `--keep-claudecode` + `--dry-run` | dry-run path | warning on stderr; dry-run output unaffected |

## Acceptance Criteria

- `run_built_command()` contains warning block checking `cli.keep_claudecode && verbosity.shows_warnings() && std::env::var("CLAUDECODE").is_ok()`
- Warning message contains both `"--keep-claudecode"` and `"CLAUDECODE"` and `"nested-agent mode"`
- T01–T06 all have corresponding passing tests in `bug_reproducers_248_test.rs`
- `w3 .test l::3` passes at zero failures/warnings
- `Fix(BUG-248)` source comment present with `Root cause:` and `Pitfall:` fields

## Validation

### Checklist

**Implementation (positive)**
- [ ] `run_built_command()` contains the warning block after `verbosity` is computed?
- [ ] Warning is gated on `verbosity.shows_warnings()` (level ≥ 2)?
- [ ] Warning text includes `"--keep-claudecode"`, `"CLAUDECODE"`, and `"nested-agent mode"`?
- [ ] `Fix(BUG-248)` comment present with all 3 fields?
- [ ] T01–T06 all have passing tests?
- [ ] `w3 .test l::3` passes at zero failures/warnings?

**Out of Scope (absence)**
- [ ] `build_claude_command()` is unmodified?
- [ ] `apply_env_vars()` is unmodified?
- [ ] Warning is NOT fatal (does not call `std::process::exit`)?

### Measurements

- Test count: ≥ 6 new passing tests (T01–T06)
- `w3 .test l::3`: 0 failures, 0 warnings

### Invariants

- Verbosity gate: `grep -A10 "BUG-248" module/claude_runner/src/cli/mod.rs` must contain `shows_warnings()`

### Anti-faking checks

- `grep -n "keep_claudecode" module/claude_runner/src/cli/mod.rs` — must show at least two references: one in `build_claude_command` (builder call) and one in `run_built_command` (warning check)

## Related Documentation

- `module/claude_runner_core/docs/failure_mode/003_claudecode_env_leak.md` — documents the root failure mode (clr Response section describes this gap)
- `module/claude_runner_core/docs/failure_mode/readme.md` — Silent Fails Table row 003 with clr Response column
- `module/claude_runner_core/docs/feature/006_unset_claudecode.md` — design rationale for unset_claudecode default-on
- `module/claude_runner/task/claude_runner/bug/248_keep_claudecode_no_warning.md` — formal bug report (BUG-248)

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | In Scope: 3 concrete bounded items; Out of Scope: 5 meaningful exclusions; observable outcome: warning appears on stderr with `CLAUDECODE=1` + `--keep-claudecode` + `w3 .test l::3` green. |
| MOST Goal Quality | PASS | Motivated (nested-agent mode silently enabled); Observable (warning line on stderr, w3 passes); Scoped (one block in `run_built_command()` + one test file); Testable (mechanical test runner + grep anti-faking checks). |
| Value / YAGNI | PASS | No null-hypothesis defense survives; concrete committed need (BUG-248 filed, failure_mode/003 documented); no scope creep. |
| Implementation Readiness | PASS | Steps numbered 1–6, ordered, unambiguous; Test Matrix 6 concrete rows covering verbosity gate; file paths and insertion point specified. |

Verified: 2026-06-07. Transition: ❓ → 🎯.

## History

- **[2026-06-07]** `CREATED` — Emit warning when `--keep-claudecode` disables CLAUDECODE protection while env var is set (BUG-248).
- **[2026-06-07]** `VERIFIED` — MAAV passed (4/4 dimensions). Transitioned ❓→🎯.
- **[2026-06-07]** `COMPLETED` — All tests pass (16/16 crates green, w3 .test l::3). Moved to completed/.
