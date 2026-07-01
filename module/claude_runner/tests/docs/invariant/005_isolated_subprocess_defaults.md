# Test: Invariant — Isolated Subprocess Defaults

Test case planning for [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md). Tests validate that `isolated` and `refresh` inject the required model, effort level, flags, and CLAUDE.md behavior in every subprocess invocation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| ISD-1 | `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"` | Model Constant |
| ISD-2 | `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"` | Model Constant |
| ISD-3 | `clr isolated --trace "x"` stderr shows `--effort max` | Effort Injection |
| ISD-4 | `clr refresh --trace` stderr shows `--effort low` | Effort Injection |
| ISD-5 | `clr isolated --trace "x"` stderr shows `--dangerously-skip-permissions` | Skip-Permissions |
| ISD-6 | `clr isolated --trace` (no message) stderr does not show `--dangerously-skip-permissions` | Skip-Permissions |
| ISD-7 | `clr isolated --trace "x"` stderr shows `--no-session-persistence` | Session Persistence |
| ISD-8 | `clr refresh --trace` stderr shows `--no-session-persistence` | Session Persistence |
| ISD-9 | `clr refresh --trace` stderr shows `--no-chrome` | Chrome Suppression |
| ISD-10 | `clr isolated --trace "x"` stderr does not show `--no-chrome` (chrome is the default) | Chrome Suppression |
| ISD-11 | CLAUDE.md written to temp HOME before spawn | CLAUDE.md Provisioning |
| ISD-12 | `clr isolated --timeout 0 "x" --trace` does not kill subprocess immediately | Timeout Semantics |
| ISD-13 | Passthrough `-- --effort medium` overrides injected `--effort max` for isolated | Passthrough Override |

## Test Coverage Summary

- Model Constant: 2 tests (ISD-1, ISD-2)
- Effort Injection: 2 tests (ISD-3, ISD-4)
- Skip-Permissions: 2 tests (ISD-5, ISD-6)
- Session Persistence: 2 tests (ISD-7, ISD-8)
- Chrome Suppression: 2 tests (ISD-9, ISD-10)
- CLAUDE.md Provisioning: 1 test (ISD-11)
- Timeout Semantics: 1 test (ISD-12)
- Passthrough Override: 1 test (ISD-13)

**Total:** 13 tests

---

### ISD-1: `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"`

- **Given:** `module/claude_runner_core/src/isolated.rs` compiled with `ISOLATED_DEFAULT_MODEL` constant
- **When:** static check of the constant value
- **Then:** `ISOLATED_DEFAULT_MODEL` equals `"claude-opus-4-6"`
- **Exit:** 0
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-2: `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"`

- **Given:** `module/claude_runner_core/src/isolated.rs` compiled with `REFRESH_DEFAULT_MODEL` constant
- **When:** static check of the constant value
- **Then:** `REFRESH_DEFAULT_MODEL` equals `"claude-sonnet-4-6"`
- **Exit:** 0
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-3: `clr isolated --trace "x"` stderr shows `--effort max`

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`); `--dry-run` used to prevent actual spawn
- **When:** `clr isolated --creds <f> --dry-run "x"` or trace inspection of args
- **Then:** trace/dry-run output contains `--effort max` before `--print` in the assembled command line
- **Exit:** 0
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-4: `clr refresh --trace` stderr shows `--effort low`

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`); trace enabled
- **When:** `clr refresh --creds <f> --trace` (no spawn; trace fires before invoke)
- **Then:** stderr contains `--effort low` in the assembled command line before `--print`
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-5: `clr isolated --trace "x"` stderr shows `--dangerously-skip-permissions`

- **Given:** credentials JSON written to a temp file `<f>`; message `"x"` provided
- **When:** `clr isolated --creds <f> --trace "x"`
- **Then:** trace stderr contains `--dangerously-skip-permissions` in the assembled command line
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-6: `clr isolated --trace` (no message) does not inject `--dangerously-skip-permissions`

- **Given:** credentials JSON written to a temp file `<f>`; no message (interactive mode)
- **When:** `clr isolated --creds <f> --trace`
- **Then:** trace stderr does NOT contain `--dangerously-skip-permissions`; subprocess enters interactive mode
- **Exit:** 1 (claude absent) or 0 (claude present; interactive)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-7: `clr isolated --trace "x"` stderr shows `--no-session-persistence`

- **Given:** credentials JSON written to a temp file `<f>`; message `"x"` provided
- **When:** `clr isolated --creds <f> --trace "x"`
- **Then:** trace stderr contains `--no-session-persistence` in the assembled command line
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-8: `clr refresh --trace` stderr shows `--no-session-persistence`

- **Given:** credentials JSON written to a temp file `<f>`
- **When:** `clr refresh --creds <f> --trace`
- **Then:** trace stderr contains `--no-session-persistence` in the assembled command line
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-9: `clr refresh --trace` stderr shows `--no-chrome`

- **Given:** credentials JSON written to a temp file `<f>`
- **When:** `clr refresh --creds <f> --trace`
- **Then:** trace stderr contains `--no-chrome` in the assembled command line; `--chrome` is suppressed for refresh
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-10: `clr isolated --trace "x"` does not show `--no-chrome`

- **Given:** credentials JSON written to a temp file `<f>`; message `"x"` provided
- **When:** `clr isolated --creds <f> --trace "x"`
- **Then:** trace stderr does NOT contain `--no-chrome`; chrome is active (ClaudeCommand default) for isolated tasks
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-11: CLAUDE.md written to temp HOME before spawn

- **Given:** `run_isolated()` called with a valid temp HOME directory; subprocess about to be spawned
- **When:** inspect the temp HOME directory after CLAUDE.md write step but before subprocess starts (unit-level test or via static verification in `run_isolated()`)
- **Then:** `<temp_home>/.claude/CLAUDE.md` exists and contains `# Isolated subprocess` header and all five behavioral directives from `invariant/005`; file is written before `Command::spawn()` is called
- **Exit:** 0
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### ISD-12: `--timeout 0` does not kill subprocess immediately

- **Given:** `run_isolated()` called with `timeout_secs = 0`; a subprocess that pauses briefly before exiting
- **When:** subprocess sleeps 1s then exits 0
- **Then:** subprocess is not killed; exit code 0 is returned; watchdog is disabled when `timeout_secs == 0`
- **Exit:** 0
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md), [param/020_timeout.md](../../../../docs/cli/param/020_timeout.md)

---

### ISD-13: Passthrough `-- --effort medium` overrides injected `--effort max`

- **Given:** credentials JSON written to a temp file `<f>`; passthrough `-- --effort medium` provided
- **When:** `clr isolated --creds <f> --trace "x" -- --effort medium`
- **Then:** trace stderr shows `--effort max` injected first, then `--effort medium` after `--` — claude binary last-wins semantics cause `medium` to be effective; both flags appear in the command line in injection-before-passthrough order
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md), [`command_defaults.md` Injection Order Convention](../../../../docs/cli/command_defaults.md)
