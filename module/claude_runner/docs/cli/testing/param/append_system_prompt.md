# Test: `--append-system-prompt`

Edge case coverage for the `--append-system-prompt` parameter. See [params.md](../../params.md#parameter--15---append-system-prompt) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `--append-system-prompt "text"` → flag and value in dry-run output | Happy Path |
| TC-02 | `--append-system-prompt` without value → exit 1 | Missing Value |
| TC-03 | `--append-system-prompt ""` (empty string) → forwarded to claude | Empty Value |
| TC-04 | `--append-system-prompt` + `--system-prompt` together → both forwarded | Interaction |
| TC-05 | `--help` output contains `--append-system-prompt` | Documentation |

## Test Coverage Summary

- Happy Path: 1 test
- Missing Value: 1 test
- Empty Value: 1 test
- Interaction: 1 test
- Documentation: 1 test

**Total:** 5 edge cases

---

### TC-01: `--append-system-prompt "text"` → appears in dry-run output

**Goal:** `--append-system-prompt` and its value are forwarded to the assembled claude command.
**Setup:** None.
**Command:** `clr --dry-run --append-system-prompt "Always JSON." "test"`
**Expected Output:** Command line contains `--append-system-prompt` and `Always JSON.`.
**Verification:** `output.contains("--append-system-prompt")` and `output.contains("Always JSON.")`.
**Pass Criteria:** Exit 0; flag and value both present.
**Source:** [params.md — --append-system-prompt](../../params.md#parameter--15---append-system-prompt)

---

### TC-02: `--append-system-prompt` without value → exit 1

**Goal:** `--append-system-prompt` at end of argv (no following token) is a usage error.
**Setup:** None.
**Command:** `clr --append-system-prompt`
**Expected Output:** Exit code 1; stderr contains "--append-system-prompt requires a value".
**Verification:** Exit code 1.
**Pass Criteria:** Exit 1; error message shown.
**Source:** [params.md — --append-system-prompt validation](../../params.md#parameter--15---append-system-prompt)

---

### TC-03: `--append-system-prompt ""` → forwarded without rejection

**Goal:** Empty string is a valid (if unusual) append; not rejected by the runner.
**Setup:** None.
**Command:** `clr --dry-run --append-system-prompt "" "test"`
**Expected Output:** Exit 0; command assembled (empty append forwarded).
**Verification:** Exit 0.
**Pass Criteria:** Exit 0; no rejection of empty string.
**Source:** [params.md — --append-system-prompt](../../params.md#parameter--15---append-system-prompt)

---

### TC-04: `--append-system-prompt` + `--system-prompt` together → both forwarded

**Goal:** Both flags can appear in the same invocation; both are forwarded to claude.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
**Expected Output:** Output contains both `--system-prompt` and `--append-system-prompt`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; both flags present.
**Source:** [parameter_interactions.md — system prompt combinations](../../parameter_interactions.md)

---

### TC-05: `--help` lists `--append-system-prompt`

**Goal:** `--append-system-prompt` is documented in help output so users can discover it.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Stdout contains `--append-system-prompt`.
**Verification:** `output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; flag present in help.
**Source:** [commands.md — help](../../commands.md#command--2-help)
