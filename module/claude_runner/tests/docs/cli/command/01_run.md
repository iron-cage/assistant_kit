# Test: `run`

Integration test planning for the `run` command. See [command/01_run.md](../../../../docs/cli/command/01_run.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Message → print mode default, exit 0 | Happy Path |
| IT-2 | `--interactive "msg"` → no `--print` in command | Mode Selection |
| IT-3 | `--new-session` → no `-c` in assembled command | Session Control |
| IT-4 | `--dry-run` → command preview, no execution | Preview Mode |
| IT-5 | `--trace "msg"` → command on stderr before execution attempt | Trace Mode |
| IT-6 | `--system-prompt "text"` → flag forwarded to claude | System Prompt |
| IT-7 | `--append-system-prompt "text"` → flag forwarded to claude | System Prompt |
| IT-8 | Unknown flag → exit 1, error message | Error Handling |
| IT-9 | Message → suffixed with `"\n\nultrathink"` by default | Ultrathink Default |
| IT-10 | `--no-ultrathink "msg"` → message sent verbatim | Ultrathink Opt-Out |
| IT-11 | Empty string after `--` separator `-- ""` → no message (treated as bare `clr`) | Edge Case |
| IT-12 | Empty string positional `""` → no message (treated as bare `clr`) | Edge Case |
| IT-13 | Default → `--effort max` in assembled command | Effort Default |
| IT-14 | `--no-effort-max` → no `--effort` in assembled command | Effort Opt-Out |
| IT-15 | Default → `--dangerously-skip-permissions` injected | Default Injection |
| IT-16 | `--effort invalid` → exit 1, error message | Error Handling |
| IT-17 | `clr run "msg"` → identical to `clr "msg"` (BUG-212 coverage) | Explicit run alias |
| IT-18 | Empty `--session-dir` → no `-c` in assembled command (BUG-214 regression) | First-use edge case |
| IT-19 | `--subdir NAME` → dry-run effective dir ends with `/-NAME` | Subdir |
| IT-20 | Print mode subprocess exits 42 → clr exits 42 (BUG-239 regression) | Exit Code Passthrough |
| IT-21 | SIGTERM-killed subprocess → clr exits 143 (128+15) (BUG-242, unix-only) | Exit Code Passthrough |
| IT-22 | Binary not found → stderr contains install hint (BUG-241) | Error Diagnostics |

## Test Coverage Summary

- Happy Path: 1 test
- Mode Selection: 1 test
- Session Control: 1 test
- Preview Mode: 1 test
- Trace Mode: 1 test
- System Prompt: 2 tests
- Error Handling: 2 tests
- Ultrathink Default: 1 test
- Ultrathink Opt-Out: 1 test
- Edge Case: 2 tests
- Effort Default: 1 test
- Effort Opt-Out: 1 test
- Default Injection: 1 test
- Explicit run alias: 1 test
- First-use edge case: 1 test
- Subdir: 1 test
- Exit Code Passthrough: 2 tests (IT-20, IT-21)
- Error Diagnostics: 1 test (IT-22)

**Total:** 22 tests

---

### IT-1: Message → print mode default

- **Command:** `clr --dry-run "Fix bug"`
- **Expected behavior:** Command line contains `--print`
- **Exit:** 0
- **Source:** [command/01_run.md](../../../../docs/cli/command/01_run.md), [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-2: `--interactive "msg"` → no `--print`

- **Command:** `clr --dry-run --interactive "Fix bug"`
- **Expected behavior:** Command line does NOT contain `--print`
- **Exit:** 0
- **Source:** [--interactive](../../../../docs/cli/param/006_interactive.md)

---

### IT-3: `--new-session` → no `-c`

- **Command:** `clr --dry-run --new-session "Fix bug"`
- **Expected behavior:** Command line does NOT contain ` -c`
- **Exit:** 0
- **Source:** [--new-session](../../../../docs/cli/param/007_new_session.md)

---

### IT-4: `--dry-run` → preview only

- **Command:** `clr --dry-run "test" --model sonnet`
- **Expected behavior:** Env vars and command line on stdout; exit 0
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md)

---

### IT-5: `--trace "msg"` → command on stderr then execute

- **Command:** `clr --trace "Fix bug"` (no `--dry-run`; claude unavailable in test environment)
- **Expected behavior:** Stderr contains assembled command before invocation; stdout is empty or shows error from failed subprocess launch
- **Exit:** 1 (claude absent in test environment)
- **Source:** [--trace](../../../../docs/cli/param/013_trace.md)

---

### IT-6: `--system-prompt "text"` → forwarded to claude

- **Command:** `clr --dry-run --system-prompt "Be concise." "Fix bug"`
- **Expected behavior:** Command line contains `--system-prompt` and `Be concise.`
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/015_system_prompt.md)

---

### IT-7: `--append-system-prompt "text"` → forwarded to claude

- **Command:** `clr --dry-run --append-system-prompt "Always JSON." "Fix bug"`
- **Expected behavior:** Command line contains `--append-system-prompt` and `Always JSON.`
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md)

---

### IT-8: Unknown flag → exit 1

- **Command:** `clr --unknown-flag "Fix bug"`
- **Expected behavior:** Stderr contains "unknown option"; exit code 1
- **Exit:** 1
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-9: Message → suffixed with `"\n\nultrathink"` by default

- **Command:** `clr --dry-run "Fix the auth bug"`
- **Expected behavior:** Command line contains `"Fix the auth bug"` followed by `ultrathink` as suffix (not `"ultrathink Fix the auth bug"`)
- **Exit:** 0
- **Source:** [--no-ultrathink](../../../../docs/cli/param/014_no_ultrathink.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-10: `--no-ultrathink` → message sent verbatim

- **Command:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
- **Expected behavior:** Command line contains `"Fix the auth bug"` (not followed by `ultrathink`)
- **Exit:** 0
- **Source:** [--no-ultrathink](../../../../docs/cli/param/014_no_ultrathink.md)

---

### IT-11: Empty string after `--` separator → no message

- **Command:** `clr --dry-run -- ""`
- **Expected behavior:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg)
- **Exit:** 0
- **Source:** fix issue-empty-msg-double-dash

---

### IT-12: Empty string positional `""` → no message

- **Command:** `clr --dry-run ""`
- **Expected behavior:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg)
- **Exit:** 0
- **Source:** fix issue-empty-msg-ultrathink

---

### IT-13: Default → `--effort max` in assembled command

- **Command:** `clr --dry-run "Fix bug"`
- **Expected behavior:** Command line contains `--effort max`
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md), [--effort](../../../../docs/cli/param/017_effort.md)

---

### IT-14: `--no-effort-max` → no `--effort` in assembled command

- **Command:** `clr --dry-run --no-effort-max "Fix bug"`
- **Expected behavior:** Command line does NOT contain `--effort`
- **Exit:** 0
- **Source:** [--no-effort-max](../../../../docs/cli/param/018_no_effort_max.md)

---

### IT-15: Default → `--dangerously-skip-permissions` injected

- **Command:** `clr --dry-run "Fix bug"`
- **Expected behavior:** Command line contains `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md), [--no-skip-permissions](../../../../docs/cli/param/005_no_skip_permissions.md)

---

### IT-16: `--effort invalid` → exit 1

- **Command:** `clr --effort bad_value "Fix bug"`
- **Expected behavior:** Stderr contains error message listing valid values (`low`, `medium`, `high`, `max`); exit code 1
- **Exit:** 1
- **Source:** [type/07_effort_level.md](../../../../docs/cli/type/07_effort_level.md)

---

### IT-17: `clr run "msg"` → identical assembled command to `clr "msg"` (BUG-212 coverage)

- **Command:** `clr --dry-run run "Fix bug"` vs `clr --dry-run "Fix bug"`
- **Expected behavior:** Both produce an identical assembled command line; the leading `run` token is stripped before parsing; `"Fix bug"` is the message in both cases; command contains `--print` and `ultrathink` suffix
- **Exit:** 0 for both
- **Source:** [command/01_run.md](../../../../docs/cli/command/01_run.md), [invariant/003_command_naming.md](../../../../docs/invariant/003_command_naming.md)

---

### IT-18: Empty `--session-dir` → no `-c` in assembled command (BUG-214 regression)

- **Command:** `clr --dry-run --session-dir /tmp/mre214_empty "Fix bug"`
- **Expected behavior:** Assembled command does NOT include `-c`; `session_exists()` guard detects empty directory and suppresses injection
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md § Fixed Defects](../../../../docs/invariant/001_default_flags.md), [--session-dir](../../../../docs/cli/param/010_session_dir.md)

---

### IT-19: `--subdir NAME` → effective dir ends with `/-NAME`

- **Command:** `clr --dry-run --subdir build "Fix bug"`
- **Expected behavior:** Dry-run output contains a path ending in `/-build`; exit 0
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md), [user_story/022_session_isolation_subdir.md](../../../../docs/cli/user_story/022_session_isolation_subdir.md)

---

### IT-20: Print mode subprocess exits 42 → clr exits 42 (BUG-239 regression)

- **Setup:** Fake-claude script that unconditionally exits 42; script injected via PATH
- **Command:** `clr --print "test"` with fake-claude in PATH
- **Expected behavior:** `clr` exits 42; exit code is propagated exactly from the subprocess, not hardcoded to 1
- **Exit:** 42
- **Source:** [command/01_run.md — Exit Codes table](../../../../docs/cli/command/01_run.md)
- **Note:** Implemented in TSK-196 (BUG-239); test function `print_mode_propagates_exit_42` in `tests/bug_reproducers_239_244_test.rs`

---

### IT-21: SIGTERM-killed subprocess → clr exits 143 (128+15) (BUG-242, unix-only)

- **Setup:** Fake-claude script that sleeps indefinitely; script injected via PATH
- **Command:** `clr --print "test"` with sleeping fake-claude; SIGTERM sent to subprocess PID
- **Expected behavior:** `clr` exits 143 (= 128 + 15); signal-killed exit codes follow POSIX 128+signal convention
- **Exit:** 143
- **Source:** [command/01_run.md — Exit Codes table](../../../../docs/cli/command/01_run.md)
- **Platform:** `#[cfg(unix)]` — not applicable on Windows
- **Note:** Implemented in TSK-196 (BUG-242); test function `signal_sigterm_exits_143` in `tests/bug_reproducers_239_244_test.rs`

---

### IT-22: Binary not found → stderr contains install hint (BUG-241)

- **Setup:** PATH set to a directory containing no `claude` binary; `CLR_CLAUDE_BIN` unset
- **Command:** `clr "Fix bug"` (or `clr --print "Fix bug"`)
- **Expected behavior:** stderr contains both `"not found"` and `"install"`; user-actionable install hint shown instead of raw OS error
- **Exit:** 1
- **Source:** [command/01_run.md](../../../../docs/cli/command/01_run.md), [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)
- **Note:** Implemented in TSK-196 (BUG-241); test function `binary_not_found_shows_install_hint` in `tests/bug_reproducers_239_244_test.rs`
