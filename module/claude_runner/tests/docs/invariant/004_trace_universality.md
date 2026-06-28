# Test: Invariant — Trace Universality

Test case planning for [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md). Tests validate that every subprocess-executing command accepts `--trace` and produces diagnostic output on stderr before invocation.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `clr --trace "msg"` (run) → stderr contains env + command | Run Trace |
| IN-2 | `clr ask --trace "msg"` → stderr identical to run trace (pure alias) | Ask Trace |
| IN-3 | `clr isolated --creds <f> --trace "msg"` → stderr contains `# clr isolated` / `# creds:` / `# timeout: 30s` | Isolated Trace |
| IN-4 | `clr refresh --creds <f> --trace` → stderr contains `# clr refresh` / `# creds:` / `# timeout: 45s` | Refresh Trace |
| IN-5 | Static: `--trace` parsed by all four subprocess-executing commands | Structural Invariant |

## Test Coverage Summary

- Run Trace: 1 test (IN-1)
- Ask Trace: 1 test (IN-2)
- Isolated Trace: 1 test (IN-3)
- Refresh Trace: 1 test (IN-4)
- Structural Invariant: 1 test (IN-5)

**Total:** 5 tests

---

### IN-1: `clr --trace "msg"` (run) → stderr contains env + command

- **Given:** clean environment; claude binary absent in test environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`)
- **Then:** stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` and the assembled `env -u CLAUDECODE claude --dangerously-skip-permissions -c --print "Fix bug\n\nultrathink"` command line before invocation attempt; `--chrome` is absent (print mode — auto-suppressed per BUG-304); the `env -u CLAUDECODE` prefix reflects the default `unset_claudecode=true` (BUG-246 WYSIWYG fix); exit 1 (claude absent)
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [cli/param/013_trace.md](../../../../docs/cli/param/013_trace.md)

---

### IN-2: `clr ask --trace "msg"` → stderr identical to run trace (pure alias)

- **Given:** clean environment; claude binary absent in test environment
- **When:** `clr ask --trace "What is X?"`
- **Then:** stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` and the assembled `env -u CLAUDECODE claude --dangerously-skip-permissions --effort max --print -c "What is X?\n\nultrathink"` command line — identical to `clr --trace "What is X?"` output since ask is a pure semantic alias; `--chrome` is absent (print mode — auto-suppressed per BUG-304); exit 1 (claude absent)
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IN-3: `clr isolated --creds <f> --trace "msg"` → stderr contains credential trace

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`)
- **When:** `clr isolated --creds <f> --trace "Fix bug"` (no `--dry-run`; trace fires before creds file read, so output appears on stderr regardless of whether the file is readable)
- **Then:** stderr contains `# clr isolated`, `# creds: <path>`, `# timeout: 30s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --chrome --model claude-opus-4-6 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"` before any subprocess attempt; `--chrome` is visible because `emit_credential_trace()` uses `ClaudeCommand::new()` default (not affected by builder.rs BUG-304 print-mode suppression); exit 0 or 1
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### IN-4: `clr refresh --creds <f> --trace` → stderr contains credential trace with 45s timeout

- **Given:** credentials JSON written to a temp file `<f>` (file is readable; content `{}`)
- **When:** `clr refresh --creds <f> --trace` (no `--dry-run`; trace fires before creds file read)
- **Then:** stderr contains `# clr refresh`, `# creds: <path>`, `# timeout: 45s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --model claude-sonnet-4-6 --no-chrome --effort low --no-session-persistence --print "."` before any subprocess attempt; exit 0 or 1
- **Exit:** 1 (claude absent) or 0 (claude present)
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [command/03_refresh.md](../../../../docs/cli/command/03_refresh.md), [invariant/005_isolated_subprocess_defaults.md](../../../../docs/invariant/005_isolated_subprocess_defaults.md)

---

### IN-5: Static: `--trace` parsed by all four subprocess-executing commands

- **Given:** static analysis of `src/cli/parse.rs` and `src/cli/cred_parse.rs`
- **When:** inspect `parse_args()` in `src/cli/parse.rs`; `parse_isolated_args()` and `parse_refresh_args()` in `src/cli/cred_parse.rs` (and `dispatch_ask()` which calls `parse_args()`)
- **Then:** all four functions include `--trace` in their flag definitions; no subprocess-executing command omits it; `help` does not accept `--trace` (it is not a subprocess-executing command)
- **Exit:** 0
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
