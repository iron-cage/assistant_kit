# Test: `run`

Integration test planning for the `run` command. See [commands.md](../../../../docs/cli/commands.md#command--1-run) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Message → print mode default, exit 0 | Happy Path |
| IT-2 | `--interactive "msg"` → no `--print` in command | Mode Selection |
| IT-3 | `--new-session` → no `-c` in assembled command | Session Control |
| IT-4 | `--dry-run` → command preview, no execution | Preview Mode |
| IT-5 | `--trace "msg"` → command on stderr, then execute | Trace Mode |
| IT-6 | `--system-prompt "text"` → flag forwarded to claude | System Prompt |
| IT-7 | `--append-system-prompt "text"` → flag forwarded to claude | System Prompt |
| IT-8 | Unknown flag → exit 1, error message | Error Handling |
| IT-9 | Message → suffixed with `"\n\nultrathink"` by default | Ultrathink Default |
| IT-10 | `--no-ultrathink "msg"` → message sent verbatim | Ultrathink Opt-Out |
| IT-12 | Empty string positional `""` → no message (treated as bare `clr`) | Edge Case |
| IT-11 | Empty string after `--` separator `-- ""` → no message (treated as bare `clr`) | Edge Case |
| IT-13 | Default → `--effort max` in assembled command | Effort Default |
| IT-14 | `--no-effort-max` → no `--effort` in assembled command | Effort Opt-Out |

## Test Coverage Summary

- Happy Path: 1 test
- Mode Selection: 1 test
- Session Control: 1 test
- Preview Mode: 1 test
- Trace Mode: 1 test
- System Prompt: 2 tests
- Error Handling: 1 test
- Ultrathink Default: 1 test
- Ultrathink Opt-Out: 1 test
- Edge Case: 2 tests
- Effort Default: 1 test
- Effort Opt-Out: 1 test

**Total:** 14 tests

---

### IT-1: Message → print mode default

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line contains `--print`.; `--print` present in assembled command
- **Exit:** 0
- **Source:** [commands.md — run](../../../../docs/cli/commands.md#command--1-run), [feature/001_runner_tool.md](../../../../docs/feature/001_runner_tool.md)

---

### IT-2: `--interactive "msg"` → no `--print`

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Command line does NOT contain `--print`.; `--print` absent from assembled command
- **Exit:** 0
- **Source:** [params.md — --interactive](../../../../docs/cli/params.md#parameter--6---interactive)

---

### IT-3: `--new-session` → no `-c`

- **Given:** clean environment
- **When:** `clr --dry-run --new-session "Fix bug"`
- **Then:** Command line does NOT contain ` -c`.; `-c` absent from assembled command
- **Exit:** 0
- **Source:** [params.md — --new-session](../../../../docs/cli/params.md#parameter--7---new-session)

---

### IT-4: `--dry-run` → preview only

- **Given:** clean environment
- **When:** `clr --dry-run "test" --model sonnet`
- **Then:** Env vars and command line on stdout; exit 0.; no subprocess launched; output present
- **Exit:** 0
- **Source:** [params.md — --dry-run](../../../../docs/cli/params.md#parameter--11---dry-run)

---

### IT-5: `--trace "msg"` → command on stderr then execute

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (with claude unavailable)
- **Then:** Stderr contains assembled command; may exit non-zero if claude absent.; Stderr has command preview before execution attempt
- **Exit:** 0
- **Source:** [params.md — --trace](../../../../docs/cli/params.md#parameter--13---trace)

---

### IT-6: `--system-prompt "text"` → forwarded to claude

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise." "Fix bug"`
- **Then:** Command line contains `--system-prompt` and `Be concise.`.; flag and value in assembled command
- **Exit:** 0
- **Source:** [params.md — --system-prompt](../../../../docs/cli/params.md#parameter--15---system-prompt)

---

### IT-7: `--append-system-prompt "text"` → forwarded to claude

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always JSON." "Fix bug"`
- **Then:** Command line contains `--append-system-prompt` and `Always JSON.`.; flag and value in assembled command
- **Exit:** 0
- **Source:** [params.md — --append-system-prompt](../../../../docs/cli/params.md#parameter--16---append-system-prompt)

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
- **Source:** [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink), [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md)

---

### IT-10: `--no-ultrathink` → message sent verbatim

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
- **Then:** Command line contains `"Fix the auth bug"` (not followed by `ultrathink`).; message verbatim, no ultrathink suffix
- **Exit:** 0
- **Source:** [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink)

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
- **Source:** [invariant/001_default_flags.md](../../../../docs/invariant/001_default_flags.md), [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort)

---

### IT-14: `--no-effort-max` → no `--effort` in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --no-effort-max "Fix bug"`
- **Then:** Command line does NOT contain `--effort`.; suppression applied; no effort flag forwarded
- **Exit:** 0
- **Source:** [params.md — --no-effort-max](../../../../docs/cli/params.md#parameter--18---no-effort-max)
