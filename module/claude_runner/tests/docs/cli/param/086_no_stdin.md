# Param :: `--no-stdin`

Edge case tests for the `--no-stdin` flag (env: `CLR_NO_STDIN`), which opts out
of all stdin reading — both stdin JSON config auto-detection and stdin content
forwarding — before the pre-parse blocking read can occur (BUG-492).

**Source:** [param/086_no_stdin.md](../../../../docs/cli/param/086_no_stdin.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--no-stdin` unblocks a held-open non-TTY pipe | Guard |
| EC-2 | `CLR_NO_STDIN=1` env var equivalent to the flag | EnvFallback |
| EC-3 | `--no-stdin` declines piped stdin JSON config | Guard |

## Test Coverage Summary

- Guard: 2 tests (EC-1, EC-3)
- EnvFallback: 1 test (EC-2)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: `--no-stdin` unblocks a held-open non-TTY pipe

- **Given:** clr's stdin is a pipe whose write end is held open by the test (never closed, no EOF); stub claude binary in PATH
- **When:** `clr --no-stdin --dry-run "task"` with that stdin
- **Then:** clr completes well within the test's watchdog window instead of blocking forever on the stdin read; dry-run output contains the claude invocation — without the flag, the unconditional `read_to_end` would hang until the writer closes
- **Exit:** 0
- **Source:** [param/086_no_stdin.md](../../../../docs/cli/param/086_no_stdin.md)
- **Implemented by:** `bug_reproducers_490_492_test.rs::t492_no_stdin_flag_unblocks_held_open_pipe`

---

### EC-2: `CLR_NO_STDIN=1` env var equivalent to the flag

- **Given:** same held-open-pipe stdin as EC-1; `CLR_NO_STDIN=1` in the environment; no `--no-stdin` on the CLI
- **When:** `clr --dry-run "task"` with that stdin
- **Then:** clr completes well within the watchdog window — the env route is checked in the same pre-parse Gate 0 scan as the flag
- **Exit:** 0
- **Source:** [param/086_no_stdin.md](../../../../docs/cli/param/086_no_stdin.md)
- **Implemented by:** `bug_reproducers_490_492_test.rs::t492_env_clr_no_stdin_equivalent`

---

### EC-3: `--no-stdin` declines piped stdin JSON config

- **Given:** clr's stdin is a closed pipe carrying `{"model": "<marker>"}` (valid stdin JSON config that would normally be auto-detected and applied)
- **When:** `clr --no-stdin --dry-run "hi"` with that stdin
- **Then:** dry-run succeeds and the marker model does NOT appear in the planned invocation — the piped JSON config was never read, not read-and-rejected
- **Exit:** 0
- **Source:** [param/086_no_stdin.md](../../../../docs/cli/param/086_no_stdin.md)
- **Implemented by:** `bug_reproducers_490_492_test.rs::t492_no_stdin_declines_piped_json_config`
