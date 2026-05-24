# Test: `help`

Integration test planning for help output. See [command/04_help.md](../../../../docs/cli/command/04_help.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr --help` → help output, exit 0 | Happy Path |
| IT-2 | `clr -h` → same as `--help` | Alias |
| IT-3 | Help output lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max` | Completeness |
| IT-4 | `--help` anywhere in argv → help wins | Override |
| IT-5 | `--help` wins even when unknown flags are present | Override |
| IT-6 | `--help` output goes to stdout; stderr is empty | Output Stream |
| IT-7 | `-h` output is byte-identical to `--help` output | Alias |
| IT-8 | Help output is stable across repeated invocations | Stability |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 2 tests
- Completeness: 1 test
- Override: 2 tests
- Output Stream: 1 test
- Stability: 1 test

**Total:** 8 tests

---

### IT-1: `clr --help` → help output, exit 0

- **Command:** `clr --help`
- **Expected behavior:** Contains "USAGE:", "OPTIONS:", known flags
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-2: `clr -h` → same as `--help`

- **Command:** `clr -h`
- **Expected behavior:** Same as `clr --help`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-3: Help lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`

- **Command:** `clr --help`
- **Expected behavior:** Contains `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`
- **Exit:** 0
- **Source:** [--system-prompt](../../../../docs/cli/param/015_system_prompt.md), [--append-system-prompt](../../../../docs/cli/param/016_append_system_prompt.md), [--no-ultrathink](../../../../docs/cli/param/014_no_ultrathink.md), [--effort](../../../../docs/cli/param/017_effort.md), [--no-effort-max](../../../../docs/cli/param/018_no_effort_max.md)

---

### IT-4: `--help` anywhere in argv → help wins

- **Command:** `clr --model sonnet --help "Fix bug"`
- **Expected behavior:** Help output shown; not an execution
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-5: `--help` wins even when unknown flags are present

- **Command:** `clr --unknown-flag --help "msg"` and `clr --help --unknown-flag "msg"`
- **Expected behavior:** Help shown; exit 0 for both orderings; unknown flags are ignored when `--help` is present
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-6: `--help` output goes to stdout; stderr is empty

- **Command:** `clr --help`
- **Expected behavior:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-7: `-h` output is byte-identical to `--help` output

- **Command:** `clr -h` and `clr --help`
- **Expected behavior:** stdout of both invocations is byte-identical
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)

---

### IT-8: Help output is stable across repeated invocations

- **Command:** `clr --help` (run 3 times)
- **Expected behavior:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
