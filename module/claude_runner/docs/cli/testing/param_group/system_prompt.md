# Test: System Prompt (Group 3)

Interaction test planning for the System Prompt parameter group. See [parameter_groups.md](../../parameter_groups.md#group--3-system-prompt) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-01 | `--system-prompt` alone → forwarded; `--append-system-prompt` absent | Single Flag |
| TC-02 | `--append-system-prompt` alone → forwarded; `--system-prompt` absent | Single Flag |
| TC-03 | Both together → both forwarded in parse order | Combined |
| TC-04 | Neither present → claude uses default system prompt (no system-prompt flag injected) | Default Behavior |

## Test Coverage Summary

- Single Flag: 2 tests
- Combined: 1 test
- Default Behavior: 1 test

**Total:** 4 tests

---

### TC-01: `--system-prompt` alone → forwarded

**Goal:** `--system-prompt` appears in the assembled command; `--append-system-prompt` is absent.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Be concise." "test"`
**Expected Output:** Contains `--system-prompt`; does NOT contain `--append-system-prompt`.
**Verification:** `output.contains("--system-prompt")` and `!output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; only system-prompt present.
**Source:** [parameter_groups.md — System Prompt](../../parameter_groups.md#group--3-system-prompt)

---

### TC-02: `--append-system-prompt` alone → forwarded

**Goal:** `--append-system-prompt` appears in the assembled command; `--system-prompt` is absent.
**Setup:** None.
**Command:** `clr --dry-run --append-system-prompt "Always JSON." "test"`
**Expected Output:** Contains `--append-system-prompt`; does NOT contain `--system-prompt`.
**Verification:** `output.contains("--append-system-prompt")` and `!output.contains("--system-prompt")`.
**Pass Criteria:** Exit 0; only append-system-prompt present.
**Source:** [parameter_groups.md — System Prompt](../../parameter_groups.md#group--3-system-prompt)

---

### TC-03: Both together → both forwarded in parse order

**Goal:** When both flags are given, both appear in the assembled command.
**Setup:** None.
**Command:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
**Expected Output:** Contains both `--system-prompt` and `--append-system-prompt`.
**Verification:** `output.contains("--system-prompt")` and `output.contains("--append-system-prompt")`.
**Pass Criteria:** Exit 0; both flags present in output.
**Source:** [parameter_interactions.md — system prompt combinations](../../parameter_interactions.md)

---

### TC-04: Neither present → no system-prompt flag injected

**Goal:** When neither flag is given, no system-prompt flags appear in the assembled command.
**Setup:** None.
**Command:** `clr --dry-run "test"`
**Expected Output:** Does NOT contain `--system-prompt` or `--append-system-prompt`.
**Verification:** `!output.contains("--system-prompt")`.
**Pass Criteria:** Exit 0; no system-prompt flags injected by default.
**Source:** [params.md — --system-prompt default: absent](../../params.md#parameter--15---system-prompt)
