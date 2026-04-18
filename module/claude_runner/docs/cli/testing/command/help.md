# Test: `help`

Integration test planning for help output. See [commands.md](../../commands.md#command--2-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `clr --help` → help output, exit 0 | Happy Path |
| TC-02 | `clr -h` → same as `--help` | Alias |
| TC-03 | Help output lists `--system-prompt`, `--append-system-prompt`, and `--no-ultrathink` | Completeness |
| TC-04 | `--help` anywhere in argv → help wins | Override |
| TC-05 | `--help` wins even when unknown flags are present | Override |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 1 test
- Completeness: 1 test
- Override: 2 tests

**Total:** 5 tests

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

### TC-03: Help lists `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`

**Goal:** All system-prompt, ultrathink control, and effort control flags are documented in help output so users can discover them.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Contains `--system-prompt`, `--append-system-prompt`, `--no-ultrathink`, `--effort`, and `--no-effort-max`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("--append-system-prompt")` and `output.contains("--no-ultrathink")` and `output.contains("--effort")` and `output.contains("--no-effort-max")`.
**Pass Criteria:** Exit 0; all five flags present in help.
**Source:** [params.md — --system-prompt](../../params.md#parameter--15---system-prompt), [params.md — --append-system-prompt](../../params.md#parameter--16---append-system-prompt), [params.md — --no-ultrathink](../../params.md#parameter--14---no-ultrathink), [params.md — --effort](../../params.md#parameter--17---effort), [params.md — --no-effort-max](../../params.md#parameter--18---no-effort-max)

---

### TC-04: `--help` anywhere in argv → help wins

**Goal:** `--help` overrides any other flags regardless of position.
**Setup:** None.
**Command:** `clr --model sonnet --help "Fix bug"`
**Expected Output:** Help output shown; not an execution.
**Verification:** Exit 0; stdout contains "USAGE:".
**Pass Criteria:** Exit 0; help listing wins over message and other flags.
**Source:** [commands.md — help](../../commands.md#command--2-help)

---

### TC-05: `--help` wins even when unknown flags are present

**Goal:** `--help` shows USAGE and exits 0 regardless of unknown/invalid flags in argv, in any position.
**Setup:** None.
**Commands:**
- `clr --help --not-a-real-flag`
- `clr --not-a-real-flag --help`
**Expected Output:** Help shown; exit 0 for both orderings.
**Verification:** Exit 0; stdout contains "USAGE:"; no error message.
**Pass Criteria:** Exit 0; help wins — unknown flags are ignored when `--help` is present.
**Source:** [params.md — implicit `--help`](../../params.md), fix issue-help-loses-to-unknown
