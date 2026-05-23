# Test: Invariant — Trace Universality

Test case planning for [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md). Tests validate that every subprocess-executing command accepts `--trace` and produces diagnostic output on stderr before invocation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr --trace "msg"` (run) → stderr contains env + command | Run Trace |
| IT-2 | `clr ask --trace "msg"` → stderr contains ask-default trace | Ask Trace |
| IT-3 | `clr isolated --creds <f> --trace "msg"` → stderr contains `# clr isolated` / `# creds:` / `# timeout: 30s` | Isolated Trace |
| IT-4 | `clr refresh --creds <f> --trace` → stderr contains `# clr refresh` / `# creds:` / `# timeout: 45s` | Refresh Trace |
| IT-5 | Static: `--trace` parsed by all four subprocess-executing commands | Structural Invariant |

## Test Coverage Summary

- Run Trace: 1 test (IT-1)
- Ask Trace: 1 test (IT-2)
- Isolated Trace: 1 test (IT-3)
- Refresh Trace: 1 test (IT-4)
- Structural Invariant: 1 test (IT-5)

**Total:** 5 tests

---

### IT-1: `clr --trace "msg"` (run) → stderr contains env + command

- **Given:** clean environment; claude binary absent in test environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`)
- **Then:** stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` and the assembled `claude --dangerously-skip-permissions --chrome -c --print "Fix bug\n\nultrathink"` command line before invocation attempt; exit 1 (claude absent)
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [cli/param/013_trace.md](../../../../docs/cli/param/013_trace.md)

---

### IT-2: `clr ask --trace "msg"` → stderr contains ask-default trace

- **Given:** clean environment; claude binary absent in test environment
- **When:** `clr ask --trace "What is X?"`
- **Then:** stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384` and `claude --effort high --print "What is X?"` (no `-c`, no `--dangerously-skip-permissions`, no `--chrome`); exit 1 (claude absent)
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [001_command.md — ask](../../../../docs/cli/001_command.md#command--5-ask)

---

### IT-3: `clr isolated --creds <f> --trace "msg"` → stderr contains credential trace

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`)
- **When:** `clr isolated --creds <f> --trace "Fix bug"` (no `--dry-run`; `<f>` must be a real readable file so `emit_credential_trace` is reached before subprocess attempt)
- **Then:** stderr contains `# clr isolated`, `# creds: <path>`, and `# timeout: 30s` before any subprocess attempt; exit 0 or 1
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [001_command.md — isolated](../../../../docs/cli/001_command.md#command--2-isolated)

---

### IT-4: `clr refresh --creds <f> --trace` → stderr contains credential trace with 45s timeout

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`)
- **When:** `clr refresh --creds <f> --trace` (no `--dry-run`; `<f>` must be a real readable file so `emit_credential_trace` is reached before subprocess attempt)
- **Then:** stderr contains `# clr refresh`, `# creds: <path>`, and `# timeout: 45s` before any subprocess attempt; exit 0 or 1
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh)

---

### IT-5: Static: `--trace` parsed by all four subprocess-executing commands

- **Given:** static analysis of `src/lib.rs`
- **When:** inspect `parse_args()`, `parse_isolated_args()`, `parse_refresh_args()` (and `dispatch_ask()` which calls `parse_args()`)
- **Then:** all four functions include `--trace` in their flag definitions; no subprocess-executing command omits it; `help` does not accept `--trace` (it is not a subprocess-executing command)
- **Exit:** N/A (static check)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
