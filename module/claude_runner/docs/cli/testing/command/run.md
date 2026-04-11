# Test: `run`

Integration test planning for the `run` command. See [commands.md](../../commands.md#command--1-run) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | Message → print mode default, exit 0 | Happy Path |
| TC-02 | `--interactive "msg"` → no `--print` in command | Mode Selection |
| TC-03 | `--new-session` → no `-c` in assembled command | Session Control |
| TC-04 | `--dry-run` → command preview, no execution | Preview Mode |
| TC-05 | `--trace "msg"` → command on stderr, then execute | Trace Mode |
| TC-06 | `--system-prompt "text"` → flag forwarded to claude | System Prompt |
| TC-07 | `--append-system-prompt "text"` → flag forwarded to claude | System Prompt |
| TC-08 | Unknown flag → exit 1, error message | Error Handling |
| TC-09 | Message → prefixed with `"ultrathink "` by default | Ultrathink Default |
| TC-10 | `--no-ultrathink "msg"` → message sent verbatim | Ultrathink Opt-Out |
| TC-11 | Empty string positional `""` → no message (treated as bare `clr`) | Edge Case |
| TC-12 | Empty string after `--` separator `-- ""` → no message (treated as bare `clr`) | Edge Case |

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

**Total:** 12 tests

---

### TC-01: Message → print mode default

**Goal:** Providing a message activates print mode automatically; no `-p` required.
**Setup:** None.
**Command:** `clr --dry-run "Fix bug"`
**Expected Output:** Command line contains `--print`.
**Verification:** `output.contains("--print")`.
**Pass Criteria:** Exit 0; `--print` present in assembled command.
**Source:** [commands.md — run](../../commands.md#command--1-run), [feature/001_runner_tool.md](../../../feature/001_runner_tool.md)

---

### TC-02: `--interactive "msg"` → no `--print`

**Goal:** `--interactive` suppresses the auto-`--print` default.
**Setup:** None.
**Command:** `clr --dry-run --interactive "Fix bug"`
**Expected Output:** Command line does NOT contain `--print`.
**Verification:** `!output.contains("--print")`.
**Pass Criteria:** Exit 0; `--print` absent from assembled command.
**Source:** [params.md — --interactive](../../params.md#parameter--6---interactive)

---

### TC-03: `--new-session` → no `-c`

**Goal:** `--new-session` disables default session continuation; `-c` absent from command.
**Setup:** None.
**Command:** `clr --dry-run --new-session "Fix bug"`
**Expected Output:** Command line does NOT contain ` -c`.
**Verification:** `!output.contains(" -c")`.
**Pass Criteria:** Exit 0; `-c` absent from assembled command.
**Source:** [params.md — --new-session](../../params.md#parameter--7---new-session)

---

### TC-04: `--dry-run` → preview only

**Goal:** `--dry-run` emits command description and exits without launching claude.
**Setup:** None.
**Command:** `clr --dry-run "test" --model sonnet`
**Expected Output:** Env vars and command line on stdout; exit 0.
**Verification:** Contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=` and `claude `.
**Pass Criteria:** Exit 0; no subprocess launched; output present.
**Source:** [params.md — --dry-run](../../params.md#parameter--11---dry-run)

---

### TC-05: `--trace "msg"` → command on stderr then execute

**Goal:** `--trace` emits command to stderr before launching subprocess.
**Setup:** None.
**Command:** `clr --trace "Fix bug"` (with claude unavailable)
**Expected Output:** Stderr contains assembled command; may exit non-zero if claude absent.
**Verification:** Stderr contains `claude --dangerously-skip-permissions`.
**Pass Criteria:** Stderr has command preview before execution attempt.
**Source:** [params.md — --trace](../../params.md#parameter--13---trace)

---

### TC-06: `--system-prompt "text"` → forwarded to claude

**Goal:** `--system-prompt` value appears in the assembled claude command.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Be concise." "Fix bug"`
**Expected Output:** Command line contains `--system-prompt` and `Be concise.`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("Be concise.")`.
**Pass Criteria:** Exit 0; flag and value in assembled command.
**Source:** [params.md — --system-prompt](../../params.md#parameter--15---system-prompt)

---

### TC-07: `--append-system-prompt "text"` → forwarded to claude

**Goal:** `--append-system-prompt` value appears in the assembled claude command.
**Setup:** None.
**Command:** `clr --dry-run --append-system-prompt "Always JSON." "Fix bug"`
**Expected Output:** Command line contains `--append-system-prompt` and `Always JSON.`.
**Verification:** `output.contains("--append-system-prompt")` and `output.contains("Always JSON.")`.
**Pass Criteria:** Exit 0; flag and value in assembled command.
**Source:** [params.md — --append-system-prompt](../../params.md#parameter--16---append-system-prompt)

---

### TC-08: Unknown flag → exit 1

**Goal:** Unrecognized flags are rejected with exit code 1 and an error message.
**Setup:** None.
**Command:** `clr --unknown-flag "Fix bug"`
**Expected Output:** Stderr contains "unknown option"; exit code 1.
**Verification:** Exit code 1; stderr contains "unknown option".
**Pass Criteria:** Exit 1; error message shown.
**Source:** [feature/001_runner_tool.md](../../../feature/001_runner_tool.md)

---

### TC-09: Message → prefixed with `"ultrathink "` by default

**Goal:** `clr` prepends `"ultrathink "` to every message by default, activating Claude's extended thinking mode.
**Setup:** None.
**Command:** `clr --dry-run "Fix the auth bug"`
**Expected Output:** Command line contains `"ultrathink Fix the auth bug"` (not bare `"Fix the auth bug"`).
**Verification:** `output.contains("\"ultrathink Fix the auth bug\"")`.
**Pass Criteria:** Exit 0; message appears with ultrathink prefix in assembled command.
**Source:** [params.md — --no-ultrathink](../../params.md#parameter--14---no-ultrathink), [invariant/001_default_flags.md](../../../invariant/001_default_flags.md)

---

### TC-10: `--no-ultrathink` → message sent verbatim

**Goal:** `--no-ultrathink` suppresses the ultrathink prefix so the message is forwarded exactly as typed.
**Setup:** None.
**Command:** `clr --dry-run --no-ultrathink "Fix the auth bug"`
**Expected Output:** Command line contains `"Fix the auth bug"` (not `"ultrathink Fix the auth bug"`).
**Verification:** `output.contains("\"Fix the auth bug\"")` and `!output.contains("\"ultrathink Fix")`.
**Pass Criteria:** Exit 0; message verbatim, no ultrathink prefix.
**Source:** [params.md — --no-ultrathink](../../params.md#parameter--14---no-ultrathink)

---

### TC-12: Empty string after `--` separator → no message

**Goal:** `clr -- ""` (empty string after `--` separator) is ignored — behaves identically to bare `clr -- ` (interactive REPL, no `--print`, no message forwarded to claude).
**Setup:** None.
**Command:** `clr --dry-run -- ""`
**Expected Output:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg).
**Verification:** Exit 0; last line equals bare command; output does NOT contain `"ultrathink "` (with trailing space).
**Pass Criteria:** Exit 0; empty arg after `--` silently ignored; no degenerate prompt forwarded to claude.
**Source:** fix issue-empty-msg-double-dash

---

### TC-11: Empty string positional `""` → no message

**Goal:** An empty string positional arg is ignored — `clr ""` behaves identically to bare `clr` (interactive REPL, no `--print`, no message forwarded to claude).
**Setup:** None.
**Command:** `clr --dry-run ""`
**Expected Output:** Last line is `claude --dangerously-skip-permissions --chrome -c` (no `--print`, no message arg).
**Verification:** Exit 0; last line equals bare command; output does NOT contain `"ultrathink "` (with trailing space).
**Pass Criteria:** Exit 0; empty arg silently ignored; no degenerate prompt forwarded to claude.
**Source:** fix issue-empty-msg-ultrathink
