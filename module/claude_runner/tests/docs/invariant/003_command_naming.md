# Test: Invariant — Command Naming

Test case planning for [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md). Tests validate that commands are bare words, parameters use `--`/`-` prefix, and `KNOWN_SUBCOMMANDS` dispatch is correct.

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| IN-1 | `clr help` (bare word) → exit 0, prints usage including `ask` | Bare Word Command | ✅ |
| IN-2 | `clr --help` (parameter alias) → exit 0, same output | Parameter Alias | ✅ |
| IN-3 | `clr` (no args) → interactive REPL, not help | Default Dispatch | ✅ |
| IN-4 | `clr run "msg"` (explicit) → dispatches `run` command | Bare Word Command | ✅ |
| IN-5 | `clr unknowncmd` → exit 1, unrecognized subcommand error | Unknown Command | ✅ |
| IN-6 | `KNOWN_SUBCOMMANDS` contains all 8 commands; no entry begins with `--` | Naming Invariant | ✅ |
| IN-7 | `clr is` / `clr is it so?` — common English prefix passes through guard without error | Guard False-Positive (BUG-302) | ✅ |
| IN-8 | `clr isolat` still caught by guard after BUG-302 fix (true-positive regression) | Guard True-Positive | ✅ |

## Test Coverage Summary

- Bare Word Command: 2 tests (IN-1, IN-4)
- Parameter Alias: 1 test (IN-2)
- Default Dispatch: 1 test (IN-3)
- Unknown Command Rejection: 1 test (IN-5)
- Naming Invariant: 1 test (IN-6)
- Guard False-Positive Prevention: 1 test (IN-7)
- Guard True-Positive Regression: 1 test (IN-8)

**Total:** 8 tests

---

### IN-1: `clr help` (bare word) → exit 0, prints usage including `ask`

- **Given:** clean environment
- **When:** `clr help`
- **Then:** exit 0; stdout contains usage information listing `run`, `isolated`, `refresh`, `ask`, `help`; bare word dispatch works
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md), [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IN-2: `clr --help` (parameter alias) → exit 0, same output

- **Given:** clean environment
- **When:** `clr --help` (also: `clr -h`)
- **Then:** exit 0; stdout matches `clr help` output; `--help`/`-h` are parameter aliases that trigger identical help behavior
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IN-3: `clr` (no args) → interactive REPL, not help

- **Given:** clean environment; TTY available
- **When:** `clr` (no arguments)
- **Then:** does NOT print help; enters interactive REPL mode (dispatches `run` default with no message); help requires explicit `clr help` or `clr --help`
- **Exit:** 0 (when REPL exits)
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md), [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IN-4: `clr run "msg"` (explicit bare word) → dispatches run command

- **Given:** clean environment
- **When:** `clr run --dry-run "Fix bug"`
- **Then:** stdout contains assembled command (same as `clr --dry-run "Fix bug"`); `run` bare-word prefix accepted and dispatched to the run command
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)
- **Test:** `in4_run_subcommand_explicit_dispatch_identical_to_default` in `tests/user_story_creds_isolated_test.rs` (Fix(BUG-212))

---

### IN-5: `clr unknowncmd` → exit 1, unrecognized subcommand error

- **Given:** clean environment
- **When:** `clr unknowncmd "test"`
- **Then:** exit 1; stderr contains message indicating unrecognized subcommand or similar; `KNOWN_SUBCOMMANDS` guard rejects unknown bare words
- **Exit:** 1
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IN-6: `KNOWN_SUBCOMMANDS` contains all 8 commands; no entry begins with `--`

- **Given:** static analysis of `guard_unknown_subcommand()` dispatch in `src/cli/mod.rs`
- **When:** inspect `KNOWN_SUBCOMMANDS` constant
- **Then:** `KNOWN_SUBCOMMANDS` contains exactly 8 entries: `run`, `ask`, `isolated`, `refresh`, `help`, `ps`, `kill`, `tools`; none starts with `--` or `-`; all are bare words
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IN-7: `clr is` / `clr is it so?` — common English prefix passes through guard without error

- **Given:** clean environment; `clr` built from current source
- **When:** `clr is` and `clr is it so?` (separate invocations)
- **Then:** neither invocation exits 1 via the guard; stderr does NOT contain "unknown subcommand" or "Did you mean 'isolated'?"; both reach `dispatch_run()` (which will fail for other reasons — no live claude — but NOT from the guard)
- **Exit:** non-0 (from dispatch_run, not the guard); guard exit must not fire
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)
- **Note:** `// test_kind: bug_reproducer(BUG-302)`. "is" (len 2) is a strict prefix of "isolated" — the guard enforces a minimum-length threshold (`first.len() >= 4`) to avoid false-positive rejection of common English words. Test: `bug_reproducer_302_prefix_guard_false_positive_is` in `tests/cli_args_ext_test.rs`.

---

### IN-8: `clr isolat` still caught by guard after BUG-302 fix (true-positive regression)

- **Given:** clean environment; BUG-302 fix applied (minimum-length threshold on `starts_with` branch)
- **When:** `clr isolat`
- **Then:** exits 1; stderr contains "Did you mean 'isolated'?"; guard correctly rejects a 6-character truncation of a known subcommand
- **Exit:** 1
- **Source:** [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)
- **Note:** Regression guard — confirms `first.len() >= 4` threshold preserves true-positive behavior for truncated subcommand names (len 4+). Test: `bug_302_regression_isolat_still_caught_by_prefix` in `tests/cli_args_ext_test.rs`.
