# Test: Feature — Runner Tool

Test case planning for [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md). Tests validate runner tool behavioral contracts: default command assembly, dry-run output format, verbosity gate, trace mode, and execution delegation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Default invocation → all five defaults injected together | Combined Defaults |
| FT-2 | Dry-run output has env-vars block and assembled-command block | Output Format |
| FT-3 | `--verbosity 0 --dry-run` → dry-run output still shown (verbosity does not suppress it) | Verbosity Gate |
| FT-4 | `--trace --dry-run` → trace on stderr, dry-run preview on stdout | Trace Mode |
| FT-5 | Assembled command starts with `claude` binary (execution delegated) | Separation of Concerns |
| FT-6 | `--print` present when message is provided (mode selection default) | Mode Selection |
| FT-7 | `--verbosity 0` with missing binary → fatal error still visible on stderr | Verbosity Gate |

## Test Coverage Summary

- Combined Defaults: 1 test (FT-1)
- Output Format: 1 test (FT-2)
- Verbosity Gate: 2 tests (FT-3, FT-7)
- Trace Mode: 1 test (FT-4)
- Separation of Concerns: 1 test (FT-5)
- Mode Selection: 1 test (FT-6)

**Total:** 7 tests


---

### FT-1: Default invocation → all five defaults injected

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `-c`, `--dangerously-skip-permissions`, `--chrome`, `--effort max`, and the message has `ultrathink` suffix; all five defaults present simultaneously
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### FT-2: Dry-run output has env-vars block and assembled-command block

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stdout contains at minimum two sections: one showing environment variables set for the subprocess, and one showing the assembled `claude ...` command line
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--dry-run](../../../../docs/cli/param/011_dry_run.md)

---

### FT-3: `--verbosity 0 --dry-run` → dry-run output still shown

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 0 "Fix bug"`
- **Then:** Stdout still contains the assembled command preview; verbosity level 0 does not suppress dry-run output
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### FT-4: `--trace --dry-run` → dry-run preview on stdout; stderr is EMPTY (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Stdout contains the dry-run command preview; stderr is EMPTY (`handle_dry_run` returns before the trace output block fires)
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--trace](../../../../docs/cli/param/013_trace.md)

---

### FT-5: Assembled command starts with `claude` binary

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** The assembled command line in the dry-run output starts with `claude` (not `clr` or any other binary); execution is delegated to the claude binary
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### FT-6: `--print` present when message is provided

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`; print mode is the default when a message is supplied
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--print](../../../../docs/cli/param/002_print.md)

---

### FT-7: `--verbosity 0` with missing binary → fatal error still visible on stderr

- **Given:** `PATH` set to directory with no `claude` binary; `CLR_CLAUDE_BIN` unset
- **When:** `clr --verbosity 0 "Fix bug" 2>&1 | cat`
- **Then:** stderr is NOT empty; error message contains "not found" and "install" regardless of verbosity level 0
- **Exit:** non-zero
- **Source:** [feature/001_runner_tool.md — verbosity gate](../../../../docs/feature/001_runner_tool.md)
- **Note:** Implemented in TSK-196 (BUG-240 + BUG-241); test function `spawn_error_visible_at_verbosity_0` in `tests/execution_mode_test.rs`
