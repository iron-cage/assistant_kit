# Test: Feature — Runner Tool

Test case planning for [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md). Tests validate runner tool behavioral contracts: default command assembly, dry-run output format, verbosity gate, trace mode, and execution delegation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default invocation → all five defaults injected together | Combined Defaults |
| IT-2 | Dry-run output has env-vars block and assembled-command block | Output Format |
| IT-3 | `--verbosity 0 --dry-run` → dry-run output still shown (verbosity does not suppress it) | Verbosity Gate |
| IT-4 | `--trace --dry-run` → trace on stderr, dry-run preview on stdout | Trace Mode |
| IT-5 | Assembled command starts with `claude` binary (execution delegated) | Separation of Concerns |
| IT-6 | `--print` present when message is provided (mode selection default) | Mode Selection |

## Test Coverage Summary

- Combined Defaults: 1 test (IT-1)
- Output Format: 1 test (IT-2)
- Verbosity Gate: 1 test (IT-3)
- Trace Mode: 1 test (IT-4)
- Separation of Concerns: 1 test (IT-5)
- Mode Selection: 1 test (IT-6)

**Total:** 6 tests


---

### IT-1: Default invocation → all five defaults injected

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `-c`, `--dangerously-skip-permissions`, `--chrome`, `--effort max`, and the message has `ultrathink` suffix; all five defaults present simultaneously
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-2: Dry-run output has env-vars block and assembled-command block

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stdout contains at minimum two sections: one showing environment variables set for the subprocess, and one showing the assembled `claude ...` command line
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--dry-run](../../../../docs/cli/param/11_dry_run.md)

---

### IT-3: `--verbosity 0 --dry-run` → dry-run output still shown

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 0 "Fix bug"`
- **Then:** Stdout still contains the assembled command preview; verbosity level 0 does not suppress dry-run output
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-4: `--trace --dry-run` → dry-run preview on stdout; stderr is EMPTY (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Stdout contains the dry-run command preview; stderr is EMPTY (`handle_dry_run` returns before the trace output block fires)
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--trace](../../../../docs/cli/param/13_trace.md)

---

### IT-5: Assembled command starts with `claude` binary

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** The assembled command line in the dry-run output starts with `claude` (not `clr` or any other binary); execution is delegated to the claude binary
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-6: `--print` present when message is provided

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`; print mode is the default when a message is supplied
- **Exit:** 0
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md), [--print](../../../../docs/cli/param/02_print.md)
