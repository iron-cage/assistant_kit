# Test: `dry::`

Edge case coverage for the `dry::` parameter. See [params.md](../../../../../docs/cli/params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-300 | `dry::1` → `[dry-run]` prefix on `.version.install` | Explicit True |
| TC-311 | `dry::1` on `.processes.kill` → no kill | Explicit True |
| TC-330 | `dry::1` on `.settings.set` → no file change | Explicit True |
| TC-303 | `dry::1` wins over `force::1` | Interaction |
| TC-312 | `dry::1 force::1` on `.processes.kill` → dry wins | Interaction |
| TC-357 | `dry::1` does NOT write preference keys | Side-Effect Guard |
| EC-1 | Default (absent) resolves to `dry::0` (real action) | Default Behavior |
| EC-2 | `dry::0` explicit → same as absent | Explicit False |
| EC-3 | `dry::2` → exit 1, out of range | Invalid Value |
| EC-4 | `dry::-1` → exit 1, out of range | Invalid Value |
| EC-5 | `dry::abc` → exit 1, non-integer | Format Violation |
| EC-6 | `dry::` (empty) → exit 1 | Empty Value |
| EC-7 | `dry::` only accepted by mutation commands | Command Scope |

## Test Coverage Summary

- Explicit True: 3 tests
- Interaction (dry wins over force): 2 tests
- Side-Effect Guard: 1 test
- Default Behavior: 1 test
- Explicit False: 1 test
- Invalid Value: 2 tests
- Format Violation: 1 test
- Empty Value: 1 test
- Command Scope: 1 test

**Total:** 13 edge cases

---

### TC-300: `dry::1` → `[dry-run]` prefix

**Goal:** Dry-run mode prints preview without executing the operation.
**Setup:** None.
**Command:** `cm .version.install dry::1`
**Expected Output:** output contains `[dry-run]`; exit code 0.
**Verification:** `text.contains("[dry-run]")`.
**Pass Criteria:** Exit 0; dry-run marker present.
**Source:** [feature/004_dry_run.md](../../../../../docs/feature/004_dry_run.md)

---

### TC-303: `dry::1` wins over `force::1`

**Goal:** `dry::` takes absolute precedence; `force::` is ignored when `dry::1` is set.
**Setup:** None.
**Command:** `cm .version.install dry::1 force::1`
**Expected Output:** output contains `[dry-run]`; no install.
**Verification:** `[dry-run]` present; no side effects.
**Pass Criteria:** Exit 0; preview mode only.
**Source:** [parameter_interactions.md — dry+force precedence](../../../../../docs/cli/parameter_interactions.md)

---

### TC-357: `dry::1` does NOT write preference keys

**Goal:** Dry-run must not produce any persistent side effects.
**Setup:** `HOME=<tmp>`; settings file empty.
**Command:** `cm .version.install dry::1 version::stable`
**Expected Output:** `settings.json` has no `preferredVersionSpec` key after command.
**Verification:** File absent or lacks preference keys.
**Pass Criteria:** Exit 0; no settings written.
**Source:** [feature/004_dry_run.md](../../../../../docs/feature/004_dry_run.md)

---

### EC-1: Default (absent) → `dry::0`

**Goal:** Omitting `dry::` defaults to real action mode.
**Setup:** None.
**Command:** `cm .version.install dry::1` (compare to absent).
**Expected Output:** Behavior identical to explicit `dry::0`.
**Pass Criteria:** Default and explicit 0 produce identical behavior.
**Source:** [params.md — dry:: default: 0](../../../../../docs/cli/params.md)

---

### EC-3: `dry::2` → exit 1

**Goal:** Boolean parameters only accept 0 or 1. Value 2 is out of range.
**Setup:** None.
**Command:** `cm .version.install dry::2`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — dry:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-4: `dry::-1` → exit 1

**Goal:** Negative values are rejected for boolean parameter.
**Setup:** None.
**Command:** `cm .version.install dry::-1`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — dry:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-5: `dry::abc` → exit 1

**Goal:** Non-integer strings are rejected.
**Setup:** None.
**Command:** `cm .version.install dry::abc`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — dry:: type: Boolean (0/1)](../../../../../docs/cli/params.md)

---

### EC-6: `dry::` (empty) → exit 1

**Goal:** Empty boolean value is a usage error.
**Setup:** None.
**Command:** `cm .version.install dry::`
**Expected Output:** exit code 1; error about dry:: requiring a value.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../../../docs/feature/005_cli_design.md)

---

### EC-7: `dry::` only for mutation commands

**Goal:** Read-only commands reject `dry::` as an unknown parameter.
**Setup:** None.
**Command:** `cm .version.list dry::1`
**Expected Output:** exit code 1; "unknown parameter" or similar.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../../../docs/feature/005_cli_design.md)
