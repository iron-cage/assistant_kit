# Test: `--system-prompt`

Edge case coverage for the `--system-prompt` parameter. See [params.md](../../params.md#parameter--14---system-prompt) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `--system-prompt "text"` → flag and value in dry-run output | Happy Path |
| TC-02 | `--system-prompt` without value → exit 1 | Missing Value |
| TC-03 | `--system-prompt ""` (empty string) → forwarded to claude | Empty Value |
| TC-04 | `--system-prompt` + `--append-system-prompt` together → both forwarded | Interaction |
| TC-05 | `--help` output contains `--system-prompt` | Documentation |

## Test Coverage Summary

- Happy Path: 1 test
- Missing Value: 1 test
- Empty Value: 1 test
- Interaction: 1 test
- Documentation: 1 test

**Total:** 5 edge cases

---

### TC-01: `--system-prompt "text"` → appears in dry-run output

**Goal:** `--system-prompt` and its value are forwarded to the assembled claude command.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Be concise." "test"`
**Expected Output:** Command line contains `--system-prompt` and `Be concise.`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("Be concise.")`.
**Pass Criteria:** Exit 0; flag and value both present.
**Source:** [params.md — --system-prompt](../../params.md#parameter--14---system-prompt)

---

### TC-02: `--system-prompt` without value → exit 1

**Goal:** `--system-prompt` at end of argv (no following token) is a usage error.
**Setup:** None.
**Command:** `clr --system-prompt`
**Expected Output:** Exit code 1; stderr contains "--system-prompt requires a value".
**Verification:** Exit code 1.
**Pass Criteria:** Exit 1; error message shown.
**Source:** [params.md — --system-prompt validation](../../params.md#parameter--14---system-prompt)

---

### TC-03: `--system-prompt ""` → forwarded without rejection

**Goal:** Empty string is a valid (if unusual) system prompt; not rejected by the runner.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "" "test"`
**Expected Output:** Exit 0; command assembled (empty prompt forwarded).
**Verification:** Exit 0.
**Pass Criteria:** Exit 0; no rejection of empty string.
**Source:** [params.md — --system-prompt](../../params.md#parameter--14---system-prompt)

---

### TC-04: `--system-prompt` + `--append-system-prompt` together → both forwarded

**Goal:** Both flags can appear in the same invocation; both are forwarded to claude.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
**Expected Output:** Output contains both `--system-prompt` and `--append-system-prompt`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; both flags present.
**Source:** [parameter_interactions.md — system prompt combinations](../../parameter_interactions.md)

---

### TC-05: `--help` lists `--system-prompt`

**Goal:** `--system-prompt` is documented in help output so users can discover it.
**Setup:** None.
**Command:** `clr --help`
**Expected Output:** Stdout contains `--system-prompt`.
**Verification:** `output.contains("--system-prompt")`.
**Pass Criteria:** Exit 0; flag present in help.
**Source:** [commands.md — help](../../commands.md#command--2-help)
