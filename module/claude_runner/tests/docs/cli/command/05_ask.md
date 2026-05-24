# Test: `ask`

Integration test planning for the `ask` command. See [command/05_ask.md](../../../../docs/cli/command/05_ask.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr ask "question"` → no `-c`, no skip-permissions | Default Diff |
| IT-2 | `clr ask "question"` → `--effort high` (not `--effort max`) | Default Diff |
| IT-3 | `clr ask "question"` → `CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384` | Default Diff |
| IT-4 | `clr ask "question"` → no ultrathink suffix | Default Diff |
| IT-5 | `clr ask "question"` → no chrome flag | Default Diff |
| IT-6 | `clr ask --effort max "question"` → overrides default effort | Override |
| IT-7 | `clr ask --max-tokens 200000 "question"` → overrides default tokens | Override |
| IT-8 | Unknown flag → exit 1, error message | Error Handling |
| IT-9 | `clr ask --trace "question"` → stderr contains ask-specific trace output | Trace |

## Test Coverage Summary

- Default Diff: 5 tests (IT-1 through IT-5)
- Override: 2 tests (IT-6, IT-7)
- Error Handling: 1 test (IT-8)
- Trace: 1 test (IT-9)

**Total:** 9 tests

---

### IT-1: `clr ask "question"` → no `-c`, no skip-permissions

- **Command:** `clr ask --dry-run "What does X do?"`
- **Expected behavior:** Command line does NOT contain ` -c`; does NOT contain `--dangerously-skip-permissions`; contains `--print`; contains `--no-session-persistence`
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-2: `clr ask "question"` → effort `high`

- **Command:** `clr ask --dry-run "What does X do?"`
- **Expected behavior:** Command line contains `--effort high` (not `--effort max`)
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-3: `clr ask "question"` → max-tokens 16384

- **Command:** `clr ask --dry-run "What does X do?"`
- **Expected behavior:** Env output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384`
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-4: `clr ask "question"` → no ultrathink suffix

- **Command:** `clr ask --dry-run "What does X do?"`
- **Expected behavior:** Message argument does NOT contain `ultrathink`; message appears verbatim
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-5: `clr ask "question"` → no chrome flag

- **Command:** `clr ask --dry-run "What does X do?"`
- **Expected behavior:** Command line contains neither `--chrome` nor `--no-chrome`; chrome is absent
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-6: `clr ask --effort max "question"` → overrides default effort

- **Command:** `clr ask --dry-run --effort max "What does X do?"`
- **Expected behavior:** Command line contains `--effort max` (not `--effort high`)
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-7: `clr ask --max-tokens 200000 "question"` → overrides default tokens

- **Command:** `clr ask --dry-run --max-tokens 200000 "What does X do?"`
- **Expected behavior:** Env output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-8: Unknown flag → exit 1

- **Command:** `clr ask --unknown-flag "What does X do?"`
- **Expected behavior:** Stderr contains "unknown option"; exit code 1
- **Exit:** 1
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-9: `clr ask --trace "question"` → stderr contains ask-specific trace output

- **Setup:** clean environment; claude binary absent in test environment
- **Command:** `clr ask --trace "What is X?"` (no `--dry-run`; trace fires before invocation)
- **Expected behavior:** stderr contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384` and the assembled `claude --effort high --print "What is X?"` command line (no `-c`, no `--dangerously-skip-permissions`, no `--chrome`); subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
