# US-16 CLI Discoverability: Implement Rust Test Cases

## Execution State

- **Executor Type:** ai
- **Actor:** claude
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** null
- **Validation Date:** null

## Goal

User story 016 (CLI Discoverability) was added to `docs/cli/user_story/` and its
4-case test spec created at `tests/docs/cli/user_story/16_cli_discoverability.md`.
The Rust file `tests/user_story_test.rs` covers US-01 through US-15 but has no
functions for US-16. Add the 4 missing test functions so that all specified
acceptance criteria are machine-verifiable.

Task is complete when all three of the following are true:
1. `grep -c "us16_" tests/user_story_test.rs` → 4
2. `w3 .test level::3` → 0 failures
3. No US-01 through US-15 test functions are modified or removed

## In Scope

**US-1 — help prints usage:**
- Add `us16_1_help_prints_usage()`: invoke `clr help`; assert exit 0 and stdout
  contains usage information (e.g., "Usage" or at least one known subcommand name)

**US-2 — flag aliases identical:**
- Add `us16_2_flag_aliases_identical()`: invoke `clr -h`, `clr --help`, and
  `clr help`; assert all three produce identical stdout output

**US-3 — all subcommands listed:**
- Add `us16_3_all_subcommands_listed()`: invoke `clr help`; assert stdout contains
  each of the 5 subcommand names: run, isolated, refresh, ask, help

**US-4 — no side effects:**
- Add `us16_4_no_side_effects()`: invoke `clr help` with no credentials file and no
  existing session; assert exit 0, no subprocess spawned (stdout does not contain
  subprocess telltale output), no credential writes, and process completes without
  external network access (structurally verifiable via --dry-run or binary absence)

## Out of Scope

- Changes to `docs/cli/command/04_help.md` or any `docs/` file
- Changes to `src/` source code (help behavior already implemented)
- Modifications to US-01 through US-15 test functions
- `tests/docs/cli/user_story/16_cli_discoverability.md` spec edits

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- Follow the existing `tests/user_story_test.rs` function naming pattern:
  `us<NN>_<N>_<snake_case_description>()`
- Test functions must be annotated with `#[test]` and gated by
  `#[cfg(feature = "enabled")]` (matching existing file structure)
- Use the `run_cli` / `run_cli_with_env` helpers already present in the file
- No mocking; use real binary execution via the helpers

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read governing spec** — open `tests/docs/cli/user_story/16_cli_discoverability.md`
   and confirm the 4 test cases (US-1 through US-4) and their Given/When/Then/Exit
   assertions.

2. **Read existing patterns** — read the last 80 lines of `tests/user_story_test.rs`
   (US-15 Ask Mode functions) to understand the function structure, helper calls,
   and assertion patterns in use.

3. **Read command doc** — open `docs/cli/command/04_help.md` to confirm the exact
   command invocations supported (`help`, `-h`, `--help`) and expected exit behavior.

4. **Implement us16_1** (`help` exits 0, stdout contains usage):
   - Call `run_cli(&["help"])` (or equivalent)
   - Assert `exit_code(&out) == 0`
   - Assert stdout contains "Usage" or at least one known subcommand name

5. **Implement us16_2** (flag aliases produce identical output):
   - Call `run_cli(&["help"])`, `run_cli(&["-h"])`, `run_cli(&["--help"])`
   - Assert stdout of all three are equal (string equality)
   - Assert all three exit with code 0

6. **Implement us16_3** (all subcommands listed):
   - Call `run_cli(&["help"])`
   - Assert stdout contains each of: "run", "isolated", "refresh", "ask", "help"
   - Assert exit 0

7. **Implement us16_4** (no side effects):
   - Invoke `clr help` with no credentials and no session directory
   - Assert exit 0
   - Assert stdout does not contain subprocess invocation markers (e.g., no
     Claude process trace output)
   - Use a temporary directory as session dir to confirm no session files created

8. **Run tests** — `w3 .test level::3` → 0 failures. Fix any compilation or
   assertion errors before proceeding.

9. **Verify measurements** — confirm `grep -c "us16_" tests/user_story_test.rs` → 4.

10. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍). An independent
    validator executes the 8-step procedure per `validation.rulebook.md`.

11. **Update task state** — on validation pass, set ✅, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clr help` invoked | Binary in PATH, no credentials | Exit 0; stdout contains usage summary |
| T02 | `clr -h` and `clr --help` invoked | Binary in PATH | Stdout identical to `clr help`; all three exit 0 |
| T03 | `clr help` invoked | No prior config | Stdout contains all 5 subcommand names: run, isolated, refresh, ask, help |
| T04 | `clr help` invoked | No credentials file, temp session dir | Exit 0; no subprocess marker in stdout; no files written in session dir |

## Acceptance Criteria

- `grep -c "us16_" tests/user_story_test.rs` → 4 (exactly 4 us16_ functions)
- `w3 .test level::3` → 0 failures after adding the 4 functions
- No existing us01_ through us15_ test functions modified
- All 4 functions follow existing naming pattern `us<NN>_<N>_<description>()`
- Each function maps to exactly one spec case from `16_cli_discoverability.md`

## Validation

**Execution:** An independent validator performs this walk after SUBMIT transition.
The executor does NOT self-validate.

### Checklist

- [ ] C1 — `grep -c "us16_" tests/user_story_test.rs` → 4?
- [ ] C2 — All 4 functions are `#[test]` annotated and gated by `cfg(feature = "enabled")`?
- [ ] C3 — `w3 .test level::3` → 0 failures?
- [ ] C4 — us16_1 asserts exit 0 AND stdout contains usage content?
- [ ] C5 — us16_2 asserts stdout equality across all three invocations (`help`, `-h`, `--help`)?
- [ ] C6 — us16_3 asserts all 5 subcommand names present in stdout?
- [ ] C7 — us16_4 asserts exit 0, no subprocess marker, and no session files created?
- [ ] C8 — No modifications to us01_ through us15_ test functions?

### Measurements

- [ ] M1 — 4 us16_ functions: `grep -c "us16_" tests/user_story_test.rs` → 4
- [ ] M2 — Test suite: `w3 .test level::3` → 0 failures
- [ ] M3 — No regressions: us01–us15 function count unchanged vs. pre-task baseline

### Invariants

- [ ] I1 — All 4 functions use real binary execution (no mocks)
- [ ] I2 — Function naming: all match `us16_[1-4]_*` pattern

### Anti-faking checks

- [ ] AF1 — `grep "us16_" tests/user_story_test.rs | grep -c "#\[test\]"` → 0 is wrong;
  verify each us16_ function is preceded by `#[test]` attribute
- [ ] AF2 — Each us16_ function body contains at least one `assert` call

## Affected Entities

| Entity | Directory | Impact |
|--------|-----------|--------|
| `cli/user_story/` | `docs/cli/user_story/` | Instance 016 (CLI Discoverability) gains Rust test coverage |

## Related Documentation

**Source spec (defines what to test):**
- `tests/docs/cli/user_story/16_cli_discoverability.md` — 4 test cases US-1..US-4

**Behavioral requirement (defines what help must do):**
- `docs/cli/user_story/016_cli_discoverability.md` — acceptance criteria for help command
- `docs/cli/command/04_help.md` — help command reference

**Test surface for related command:**
- `tests/docs/cli/command/04_help.md` — command-level test spec

**Existing test file (implementation target):**
- `tests/user_story_test.rs` — covers US-01 through US-15; this task extends to US-16

**Related task:**
- `task/claude_runner/001_test_surface_remediation.md` — created the test spec
  `16_cli_discoverability.md`; Rust implementation was explicitly Out of Scope there

## History

- **[2026-05-24]** `CREATED` — Add 4 Rust test functions for US-16 (CLI Discoverability)
  following the normalization session that introduced `docs/cli/user_story/016_cli_discoverability.md`
  and `tests/docs/cli/user_story/16_cli_discoverability.md`.

## Verification Record

- **Date:** 2026-05-24
- **Dimensions checked:** Scope Coherence, MOST Goal Quality, Value/YAGNI, Implementation Readiness
- **Result:** All 4 dimensions PASS
- **Notes:** All 4 independent subagents returned PASS with no findings requiring remediation.
