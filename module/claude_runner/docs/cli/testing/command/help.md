# Test: `help`

Integration test planning for help output. See [commands.md](../../commands.md#command--2-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `clr --help` → help output, exit 0 | Happy Path |
| TC-02 | `clr -h` → same as `--help` | Alias |
| TC-03 | Help output lists `--system-prompt` and `--append-system-prompt` | Completeness |
| TC-04 | `--help` anywhere in argv → help wins | Override |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 1 test
- Completeness: 1 test
- Override: 1 test

**Total:** 4 tests

---

### TC-01: `clr --help` → help output, exit 0

**Goal:** `--help` prints usage and exits 0.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Contains "USAGE:", "OPTIONS:", known flags.
**Verification:** Exit 0; stdout contains "USAGE:".
**Pass Criteria:** Exit 0; help listing present.
**Source:** [commands.md — help](../../commands.md#command--2-help)

---

### TC-02: `clr -h` → same as `--help`

**Goal:** Short alias `-h` produces identical output to `--help`.
**Setup:** None.
**Command:** `clr -h`
**Expected Output:** Same as `clr --help`.
**Verification:** Exit 0; stdout contains "USAGE:".
**Pass Criteria:** Exit 0; help listing present.
**Source:** [commands.md — help](../../commands.md#command--2-help)

---

### TC-03: Help lists `--system-prompt` and `--append-system-prompt`

**Goal:** New system-prompt flags are documented in help output.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Contains `--system-prompt` and `--append-system-prompt`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; both flags present in help.
**Source:** [params.md — --system-prompt](../../params.md#parameter--14---system-prompt), [params.md — --append-system-prompt](../../params.md#parameter--15---append-system-prompt)

---

### TC-04: `--help` anywhere in argv → help wins

**Goal:** `--help` overrides any other flags regardless of position.
**Setup:** None.
**Command:** `clr --model sonnet --help "Fix bug"`
**Expected Output:** Help output shown; not an execution.
**Verification:** Exit 0; stdout contains "USAGE:".
**Pass Criteria:** Exit 0; help listing wins over message and other flags.
**Source:** [commands.md — help](../../commands.md#command--2-help)
