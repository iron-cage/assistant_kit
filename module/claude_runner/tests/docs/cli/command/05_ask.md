# Test: `ask`

Integration test planning for the `ask` command. See [command/05_ask.md](../../../../docs/cli/command/05_ask.md) for specification.

`ask` is a semantic alias for `run` — no behavioral differences. Tests focus on
structural equivalence and shared behavior.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr ask "q"` dry-run output identical to `clr "q"` dry-run output | Equivalence |
| IT-2 | `clr ask --dry-run` (no message) identical to `clr --dry-run` | Equivalence |
| IT-3 | Unknown flag → exit 1, error message | Error Handling |
| IT-4 | `clr ask --trace "q"` → stderr contains assembled command | Trace |
| IT-5 | `clr ask --subdir NAME "q"` → effective dir ends with `/-NAME` | Subdir |
| IT-6 | `clr ask --dry-run --effort high "q"` → contains `--effort high` | Param Passthrough |
| IT-7 | `clr ask --dry-run --model sonnet "q"` → contains `--model sonnet` | Param Passthrough |
| IT-8 | `clr ask help` → dispatches to help, exit 0 | Help |

## Test Coverage Summary

- Equivalence: 2 tests (IT-1, IT-2)
- Error Handling: 1 test (IT-3)
- Trace: 1 test (IT-4)
- Subdir: 1 test (IT-5)
- Param Passthrough: 2 tests (IT-6, IT-7)
- Help: 1 test (IT-8)

**Total:** 8 tests

---

### IT-1: `clr ask "q"` dry-run identical to `clr "q"` dry-run

- **Command:** `clr ask --dry-run "What does X do?"` vs `clr --dry-run "What does X do?"`
- **Expected behavior:** Both produce identical stdout (same assembled command, same env)
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-2: `clr ask --dry-run` (no message) identical to `clr --dry-run`

- **Command:** `clr ask --dry-run` vs `clr --dry-run` (with identical empty session dir)
- **Expected behavior:** Both produce identical stdout
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-3: Unknown flag → exit 1

- **Command:** `clr ask --unknown-flag "What does X do?"`
- **Expected behavior:** Stderr contains "unknown option"; exit code 1
- **Exit:** 1
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-4: `clr ask --trace "q"` → stderr contains assembled command

- **Setup:** clean environment; claude binary absent in test environment
- **Command:** `clr ask --trace "What is X?"` (no `--dry-run`; trace fires before invocation)
- **Expected behavior:** stderr contains the assembled `claude ... "What is X?\n\nultrathink"` command line; subprocess attempt fails (claude absent)
- **Exit:** 1
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)

---

### IT-5: `clr ask --subdir NAME "q"` → effective dir ends with `/-NAME`

- **Command:** `clr ask --dry-run --subdir feature "What is X?"`
- **Expected behavior:** Dry-run output contains a path ending in `/-feature`
- **Exit:** 0
- **Source:** [--subdir](../../../../docs/cli/param/028_subdir.md), [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-6: `--effort high` passed through correctly

- **Command:** `clr ask --dry-run --effort high "What does X do?"`
- **Expected behavior:** Command line contains `--effort high`
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-7: `--model sonnet` passed through correctly

- **Command:** `clr ask --dry-run --model sonnet "What does X do?"`
- **Expected behavior:** Command line contains `--model sonnet`
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)

---

### IT-8: `clr ask help` → dispatches to help

- **Command:** `clr ask help`
- **Expected behavior:** stdout contains usage information; exit code 0
- **Exit:** 0
- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md)
