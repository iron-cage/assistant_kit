# Test: `run`

Integration test planning for the `run` command. See [command.md](../../../../docs/cli/command.md#command--1-run) for specification.

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

**Total:** 16 tests

---

### IT-1: Message → print mode default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line contains `--print`.; `--print` present in assembled command
- **Exit:** 0
- **Source:** [command.md — run](../../../../docs/cli/command.md#command--1-run), [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-2: `--interactive "msg"` → no `--print`

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Command line does NOT contain `--print`.; `--print` absent from assembled command
- **Exit:** 0
- **Source:** [--interactive](../../../../docs/cli/param/06_interactive.md)

---

### IT-3: `--new-session` → no `-c`

- **Given:** clean environment
- **When:** `clr --dry-run --new-session "Fix bug"`
- **Then:** Command line does NOT contain ` -c`.; `-c` absent from assembled command
- **Exit:** 0
- **Source:** [--new-session](../../../../docs/cli/param/07_new_session.md)

---

### IT-4: `--dry-run` → preview only

- **Given:** clean environment
- **When:** `clr --dry-run "test" --model sonnet`
- **Then:** Env vars and command line on stdout; exit 0.; no subprocess launched; output present
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/11_dry_run.md)

---

### IT-5: `--trace "msg"` → command on stderr then execute

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`; claude unavailable in test environment)
- **Then:** Stderr contains assembled command before invocation; stdout is empty or shows error from failed subprocess launch
- **Exit:** 1 (claude absent in test environment)
- **Source:** [--trace](../../../../docs/cli/param/13_trace.md)

---

### IT-6: `--system-prompt "text"` → forwarded to claude

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise." "Fix bug"`
- **Then:** Command line contains `--system-prompt` and `Be concise.`.; flag and value in assembled command
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/15_system_prompt.md)

---

### IT-7: `--append-system-prompt "text"` → forwarded to claude

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always JSON." "Fix bug"`
- **Then:** Command line contains `--append-system-prompt` and `Always JSON.`.; flag and value in assembled command
- **Exit:** 0
- **Source:** [--append-system-prompt](../../../../docs/cli/param/16_append_system_prompt.md)

---

### IT-8: Unknown flag → exit 1

- **Given:** clean environment
- **When:** `clr --unknown-flag "Fix bug"`
- **Then:** Stderr contains "unknown option"; exit code 1.; error message shown
- **Exit:** 1
- **Source:** [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-9: Message → suffixed with `"\n\nultrathink"` by default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix the auth bug"`
- **Then:** Command line contains `"Fix the auth bug"` followed by `ultrathink` as suffix (not `"ultrathink Fix the auth bug"`).; message appears with ultrathink suffix in assembled command
- **Exit:** 0
- **Source:** [--no-ultrathink](../../../../docs/cli/param/14_no_ultrathink.md), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-10: `--no-ultrathink` → message sent verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
- **Then:** Command line contains `"Fix the auth bug"` (not followed by `ultrathink`).; message verbatim, no ultrathink suffix
- **Exit:** 0
- **Source:** [--no-ultrathink](../../../../docs/cli/param/14_no_ultrathink.md)

---

### IT-11: Empty string after `--` separator → no message

- **Given:** clean environment
- **When:** `clr --dry-run -- ""`
- **Then:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg).; empty arg after `--` silently ignored; no degenerate prompt forwarded to claude
- **Exit:** 0
- **Source:** fix issue-empty-msg-double-dash

---

### IT-12: Empty string positional `""` → no message

- **Given:** clean environment
- **When:** `clr --dry-run ""`
- **Then:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg).; empty arg silently ignored; no degenerate prompt forwarded to claude
- **Exit:** 0
- **Source:** fix issue-empty-msg-ultrathink

---

### IT-13: Default → `--effort max` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line contains `--effort max`.; `--effort max` present in assembled command
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md), [--effort](../../../../docs/cli/param/17_effort.md)

---

### IT-14: `--no-effort-max` → no `--effort` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max "Fix bug"`
- **Then:** Command line does NOT contain `--effort`.; suppression applied; no effort flag forwarded
- **Exit:** 0
- **Source:** [--no-effort-max](../../../../docs/cli/param/18_no_effort_max.md)

---

### IT-15: Default → `--dangerously-skip-permissions` injected

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line contains `--dangerously-skip-permissions`.; default injection in effect
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md), [--no-skip-permissions](../../../../docs/cli/param/05_no_skip_permissions.md)

---

### IT-16: `--effort invalid` → exit 1

- **Given:** clean environment
- **When:** `clr --effort bad_value "Fix bug"`
- **Then:** Stderr contains error message listing valid values (`low`, `medium`, `high`, `max`); exit code 1.; second rejection case confirming validation fires for bad --effort value
- **Exit:** 1
- **Source:** [type.md — EffortLevel validation](../../../../docs/cli/type.md#type--7-effortlevel)
