# Test: `help`

Integration test planning for help output. See [commands.md](../../../../docs/cli/commands.md#command--2-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr --help` → help output, exit 0 | Happy Path |
| IT-2 | `clr -h` → same as `--help` | Alias |
| IT-3 | Help output lists `--system-prompt`, `--append-system-prompt`, and `--no-ultrathink` | Completeness |
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

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Contains "USAGE:", "OPTIONS:", known flags.; help listing present
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-2: `clr -h` → same as `--help`

- **Given:** clean environment
- **When:** `clr -h`
- **Then:** Same as `clr --help`.; help listing present
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-3: Help lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Contains `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`.; all five flags present in help
- **Exit:** 0
- **Source:** [params.md — --system-prompt](../../../../docs/cli/params.md#parameter--15---system-prompt), [params.md — --append-system-prompt](../../../../docs/cli/params.md#parameter--16---append-system-prompt), [params.md — --no-ultrathink](../../../../docs/cli/params.md#parameter--14---no-ultrathink), [params.md — --effort](../../../../docs/cli/params.md#parameter--17---effort), [params.md — --no-effort-max](../../../../docs/cli/params.md#parameter--18---no-effort-max)

---

### IT-4: `--help` anywhere in argv → help wins

- **Given:** clean environment
- **When:** `clr --model sonnet --help "Fix bug"`
- **Then:** Help output shown; not an execution.; help listing wins over message and other flags
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-5: `--help` wins even when unknown flags are present

- **Given:** clean environment
- **When:** `clr --unknown-flag --help "msg"` and `clr --help --unknown-flag "msg"`
- **Then:** Help shown; exit 0 for both orderings; unknown flags are ignored when `--help` is present
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-6: `--help` output goes to stdout; stderr is empty

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-7: `-h` output is byte-identical to `--help` output

- **Given:** clean environment
- **When:** `clr -h` and `clr --help`
- **Then:** stdout of both invocations is byte-identical
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-8: Help output is stable across repeated invocations

- **Given:** clean environment
- **When:** `clr --help` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — help](../../../../docs/cli/commands.md#command--2-help)
