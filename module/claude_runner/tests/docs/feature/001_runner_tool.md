# Test: Feature — Runner Tool

### Scope

- **Purpose**: FT- test cases verifying default command assembly, dry-run output format, and execution mode gates for the `clr` runner tool.
- **Responsibility**: Acceptance criteria confirming combined default injection, dry-run preview structure, quiet/trace flag interaction, enum validation exit codes, and the concurrency gate.
- **In Scope**: default flag injection, dry-run env-vars/command blocks, `--quiet`, `--trace`, `--print` mode selection, `--output-file` dry-run skip, `--expect` mismatch exit 3, `--max-sessions` gate skip in dry-run.
- **Out of Scope**: journaling emission (-> `002_journaling_integration.md`), retry tier resolution (-> `003_retry_hierarchy.md`), JSON config loading (-> `004_json_config.md`), session path resolution (-> `005_session_path_resolution.md`).

Test case planning for [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md). Tests validate runner tool behavioral contracts: default command assembly, dry-run output format, quiet gate, trace mode, and execution delegation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Default invocation → all five defaults injected together | Combined Defaults |
| FT-2 | Dry-run output has env-vars block and assembled-command block | Output Format |
| FT-3 | `--quiet --dry-run` → dry-run output still shown (--quiet does not suppress it) | Quiet Gate |
| FT-4 | `--trace --dry-run` → trace on stderr, dry-run preview on stdout | Trace Mode |
| FT-5 | Assembled command contains `claude` binary invocation (execution delegated, not `clr`) | Separation of Concerns |
| FT-6 | `--print` present when message is provided (mode selection default) | Mode Selection |
| FT-7 | `--quiet` with missing binary → fatal error still visible on stderr | Quiet Gate |
| FT-8 | `--output-file` in dry-run → file NOT created | Output File Capture |
| FT-9 | `--expect` mismatch → exit 3 (fail strategy default) | Enum Output Validation |
| FT-10 | Gate skipped in dry-run mode | Concurrency Gate |

## Test Coverage Summary

- Combined Defaults: 1 test (FT-1)
- Output Format: 1 test (FT-2)
- Quiet Gate: 2 tests (FT-3, FT-7)
- Trace Mode: 1 test (FT-4)
- Separation of Concerns: 1 test (FT-5)
- Mode Selection: 1 test (FT-6)
- Output File Capture: 1 test (FT-8)
- Enum Output Validation: 1 test (FT-9)
- Concurrency Gate: 1 test (FT-10)

**Total:** 10 tests


---

### FT-1: Default invocation → all five defaults injected

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--dangerously-skip-permissions`, `--effort max`, and the message has `ultrathink` suffix; `--chrome` is absent (print mode — BUG-304 suppression; present only in interactive mode); `-c` is present when run from a directory with prior Claude sessions (verified separately by `default_continuation_always_present` using the project cwd, and by `t10_multiple_flags_combined` via explicit `--session-dir`)
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md), [invariant/001_default_flags.md](../../../docs/invariant/001_default_flags.md)

---

### FT-2: Dry-run output has env-vars block and assembled-command block

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stdout contains at minimum two sections: one showing environment variables set for the subprocess, and one showing the assembled `claude ...` command line
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md), [--dry-run](../../../docs/cli/param/011_dry_run.md)

---

### FT-3: `--quiet --dry-run` → dry-run output still shown

- **Given:** clean environment
- **When:** `clr --dry-run --quiet "Fix bug"`
- **Then:** Stdout still contains the assembled command preview; `--quiet` does not suppress dry-run output
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md)

---

### FT-4: `--trace --dry-run` → dry-run preview on stdout; stderr is EMPTY (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Stdout contains the dry-run command preview; stderr is EMPTY (`handle_dry_run` returns before the trace output block fires)
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md), [--trace](../../../docs/cli/param/013_trace.md)

---

### FT-5: Assembled command delegates to `claude` binary

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** The assembled command line in the dry-run output contains `claude` (the binary being invoked is `claude`, not `clr`); execution is delegated to the claude binary. The line starts with `env -u CLAUDECODE claude` by default (BUG-246 WYSIWYG fix).
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md)

---

### FT-6: `--print` present when message is provided

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`; print mode is the default when a message is supplied
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../docs/feature/001_runner_tool.md), [--print](../../../docs/cli/param/002_print.md)

---

### FT-7: `--quiet` with missing binary → fatal error still visible on stderr

- **Given:** `PATH` set to directory with no `claude` binary; `CLR_CLAUDE_BIN` unset
- **When:** `clr --quiet "Fix bug" 2>&1 | cat`
- **Then:** stderr is NOT empty; error message contains "not found" and "install" regardless of `--quiet`
- **Exit:** non-zero
- **Source:** [feature/001_runner_tool.md — quiet gate](../../../docs/feature/001_runner_tool.md)
- **Note:** Implemented in TSK-196 (BUG-240 + BUG-241); test function `spawn_error_visible_when_quiet` in `tests/bug_reproducers_239_244_test.rs`; also `e07_interactive_not_found_quiet_flag` and `e08_print_not_found_quiet_flag` in `tests/execution_mode_test.rs`

---

### FT-8: `--output-file` in dry-run → file NOT created

- **Given:** writable temporary path for output file; `--dry-run` active
- **When:** `clr --dry-run --output-file /tmp/feature_test_out.txt "task"`
- **Then:** exit 0; dry-run preview printed to stdout; file at the specified path does NOT exist (output file creation is skipped in dry-run mode)
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md — output file capture](../../../docs/feature/001_runner_tool.md), [--output-file](../../../docs/cli/param/029_output_file.md)

---

### FT-9: `--expect` mismatch → exit 3

- **Given:** fake-claude script outputs `maybe`; script injected via PATH; `--expect-strategy fail` (default)
- **When:** `clr -p --expect "yes|no" "task"` with fake-claude in PATH
- **Then:** `clr` exits 3; exit code 3 is exclusive to `--expect` mismatch and does not overlap with subprocess exit codes
- **Exit:** 3
- **Source:** [feature/001_runner_tool.md — enum output validation](../../../docs/feature/001_runner_tool.md), [--expect](../../../docs/cli/param/030_expect.md)

---

### FT-10: Gate skipped in dry-run mode

- **Given:** `--max-sessions 5` combined with `--dry-run`
- **When:** `clr --dry-run --max-sessions 5 "task"`
- **Then:** no "waiting" message on stderr; session count scan is skipped in dry-run mode; exit 0
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md — session concurrency gate](../../../docs/feature/001_runner_tool.md), [--max-sessions](../../../docs/cli/param/033_max_sessions.md)
